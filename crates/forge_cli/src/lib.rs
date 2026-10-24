//! Core library functions for Forge CLI, including TypeScript project type checking.

use forge_protocol::ScriptDiagnostic;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Locates the Node binary in project-local `.tools/node` or system PATH.
pub fn find_node_executable() -> Option<PathBuf> {
    // 1. Check relative to current working directory
    let local_node = Path::new(".tools/node/node.exe");
    if local_node.exists() {
        return Some(local_node.to_path_buf());
    }

    // 2. Check CARGO_MANIFEST_DIR or ancestor paths for .tools/node/node.exe
    let mut cur = std::env::current_dir().ok()?;
    loop {
        let candidate = cur.join(".tools/node/node.exe");
        if candidate.exists() {
            return Some(candidate);
        }
        let candidate_nix = cur.join(".tools/node/bin/node");
        if candidate_nix.exists() {
            return Some(candidate_nix);
        }
        if !cur.pop() {
            break;
        }
    }

    // 3. Fall back to standard PATH
    which("node")
}

/// Locates the TypeScript compiler script (`tsc` or `node_modules/typescript/bin/tsc`).
pub fn find_tsc_path() -> Option<PathBuf> {
    let mut cur = std::env::current_dir().ok()?;
    loop {
        let candidate = cur.join("node_modules/typescript/bin/tsc");
        if candidate.exists() {
            return Some(candidate);
        }
        let candidate_cmd = cur.join("node_modules/.bin/tsc.cmd");
        if candidate_cmd.exists() {
            return Some(candidate_cmd);
        }
        if !cur.pop() {
            break;
        }
    }

    which("tsc")
}

fn which(binary: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for p in std::env::split_paths(&path_var) {
        let exe = p.join(binary);
        if exe.is_file() {
            return Some(exe);
        }
        #[cfg(windows)]
        {
            let exe_ext = p.join(format!("{}.exe", binary));
            if exe_ext.is_file() {
                return Some(exe_ext);
            }
            let cmd_ext = p.join(format!("{}.cmd", binary));
            if cmd_ext.is_file() {
                return Some(cmd_ext);
            }
        }
    }
    None
}

/// Runs `tsc --noEmit` on the project scripts directory and returns parsed diagnostics.
pub fn run_typecheck(project_path: Option<&str>) -> Result<(bool, Vec<ScriptDiagnostic>), String> {
    let root = project_path.unwrap_or(".");
    let project_dir = Path::new(root);

    // Resolve tsconfig: check project_dir/tsconfig.json or project_dir/scripts/tsconfig.json
    let tsconfig = if project_dir.join("tsconfig.json").exists() {
        project_dir.join("tsconfig.json")
    } else if project_dir.join("scripts/tsconfig.json").exists() {
        project_dir.join("scripts/tsconfig.json")
    } else {
        project_dir.join("tsconfig.json")
    };

    let node_exe = find_node_executable();
    let tsc_path = find_tsc_path();

    let output = match (node_exe, tsc_path) {
        (Some(node), Some(tsc)) => Command::new(node)
            .arg(tsc)
            .arg("--noEmit")
            .arg("-p")
            .arg(&tsconfig)
            .output(),
        _ => {
            // Attempt invoking `npx tsc --noEmit` or `tsc --noEmit` directly from PATH
            #[cfg(windows)]
            let mut cmd = Command::new("cmd");
            #[cfg(windows)]
            cmd.args([
                "/C",
                "npx",
                "tsc",
                "--noEmit",
                "-p",
                tsconfig.to_str().unwrap_or("."),
            ]);

            #[cfg(not(windows))]
            let mut cmd = Command::new("npx");
            #[cfg(not(windows))]
            cmd.args(["tsc", "--noEmit", "-p", tsconfig.to_str().unwrap_or(".")]);

            cmd.output()
        }
    }
    .map_err(|e| format!("Failed to execute TypeScript compiler: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    let diagnostics = parse_tsc_output(&combined);
    let success = output.status.success() && diagnostics.is_empty();

    Ok((success, diagnostics))
}

/// Parses the output lines of `tsc --noEmit` into structured `ScriptDiagnostic` items.
pub fn parse_tsc_output(output: &str) -> Vec<ScriptDiagnostic> {
    let mut diagnostics = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Expected format:
        // filepath(line,col): error TS1234: message
        // or filepath(line,col): warning TS1234: message
        if let Some(open_paren) = trimmed.find('(') {
            if let Some(close_paren) = trimmed[open_paren..].find(')') {
                let close_paren_idx = open_paren + close_paren;
                let file = trimmed[..open_paren].trim().to_string();
                let coords = &trimmed[open_paren + 1..close_paren_idx];

                let mut parts = coords.split(',');
                let line_num: i64 = parts
                    .next()
                    .and_then(|l| l.trim().parse().ok())
                    .unwrap_or(0);
                let col_num: i64 = parts
                    .next()
                    .and_then(|c| c.trim().parse().ok())
                    .unwrap_or(0);

                let rest = trimmed[close_paren_idx + 1..].trim();
                let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

                let (severity, rest_after_sev) = if let Some(stripped) = rest.strip_prefix("error")
                {
                    ("error", stripped.trim())
                } else if let Some(stripped) = rest.strip_prefix("warning") {
                    ("warning", stripped.trim())
                } else {
                    ("info", rest)
                };

                let rest_after_colon = rest_after_sev
                    .strip_prefix(':')
                    .unwrap_or(rest_after_sev)
                    .trim();

                // Extract code if present (e.g. "TS2322: Type 'number' is not...")
                let (code, message) = if let Some(colon_pos) = rest_after_colon.find(':') {
                    let code_part = rest_after_colon[..colon_pos].trim();
                    let msg_part = rest_after_colon[colon_pos + 1..].trim();
                    (code_part.to_string(), msg_part.to_string())
                } else {
                    (String::new(), rest_after_colon.to_string())
                };

                diagnostics.push(ScriptDiagnostic {
                    file,
                    line: line_num,
                    column: col_num,
                    code,
                    message,
                    severity: severity.to_string(),
                });
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tsc_output() {
        let sample = r#"
templates/script-demo/scripts/player.ts(12,5): error TS2322: Type 'string' is not assignable to type 'number'.
templates/script-demo/scripts/coin.ts(4,10): warning TS6133: 'unused' is declared but its value is never read.
"#;
        let diags = parse_tsc_output(sample);
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].file, "templates/script-demo/scripts/player.ts");
        assert_eq!(diags[0].line, 12);
        assert_eq!(diags[0].column, 5);
        assert_eq!(diags[0].code, "TS2322");
        assert_eq!(diags[0].severity, "error");
        assert_eq!(
            diags[0].message,
            "Type 'string' is not assignable to type 'number'."
        );

        assert_eq!(diags[1].severity, "warning");
        assert_eq!(diags[1].code, "TS6133");
    }
}
