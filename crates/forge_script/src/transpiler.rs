//! TypeScript to JavaScript transpilation using SWC with source map generation.

use std::sync::Arc;
use swc::config::{Options, SourceMapsConfig};
use swc::Compiler;
use swc_common::errors::{ColorConfig, Handler};
use swc_common::{sync::Lrc, FileName, FilePathMapping, SourceMap as SwcSourceMap};

/// Result of transpiling a TypeScript source file.
#[derive(Debug, Clone)]
pub struct TranspiledScript {
    /// Transpiled JavaScript code.
    pub code: String,
    /// JSON string of the source map (if generated).
    pub source_map_json: Option<String>,
}

/// Transpiles TypeScript source code to JavaScript, stripping types and generating a source map.
pub fn transpile_typescript(filename: &str, source_code: &str) -> Result<TranspiledScript, String> {
    let cm = Arc::new(SwcSourceMap::new(FilePathMapping::empty()));
    let handler = Handler::with_tty_emitter(ColorConfig::Never, true, false, Some(cm.clone()));
    let compiler = Compiler::new(cm.clone());

    let fm = cm.new_source_file(
        Lrc::new(FileName::Real(filename.into())),
        source_code.to_string(),
    );

    let mut options = Options::default();
    options.source_maps = Some(SourceMapsConfig::Bool(true));
    options.config.jsc.target = Some(swc::config::JscTarget::Es2020);
    options.config.jsc.syntax = Some(swc::config::Syntax::Typescript(Default::default()));

    match compiler.process_js_file(fm, &handler, &options) {
        Ok(output) => Ok(TranspiledScript {
            code: output.code,
            source_map_json: output.map,
        }),
        Err(err) => Err(format!("SWC transpilation error in {}: {}", filename, err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile_simple_typescript() {
        let ts = r#"
            interface Player {
                name: string;
                score: number;
            }
            export function onStart(): void {
                const player: Player = { name: "Hero", score: 100 };
                forge.log.info("Hello", player.name);
            }
        "#;

        let result = transpile_typescript("test.ts", ts).expect("transpile should succeed");
        assert!(!result.code.contains("interface Player"));
        assert!(result.code.contains("onStart"));
        assert!(result.source_map_json.is_some());
    }
}
