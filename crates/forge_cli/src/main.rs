//! Developer command-line interface for Forge engine.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use clap::{Parser, Subcommand};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize, Debug)]
struct BridgeInfo {
    port: u16,
    token: String,
}

#[derive(Parser, Debug)]
#[command(name = "forge", about = "Forge engine developer command-line tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Connects to a running Forge runtime instance and invokes a JSON-RPC / MCP method
    Call {
        /// Name of the method or MCP tool to call (e.g. engine.ping, scene.get_tree, tools/list)
        method: String,

        /// JSON parameters payload (e.g. '{}' or '{"name":"hero"}')
        #[arg(default_value = "{}")]
        params: String,

        /// WebSocket port of the running bridge server (reads .forge/bridge.json if omitted)
        #[arg(long, short)]
        port: Option<u16>,

        /// Authentication token (reads .forge/bridge.json if omitted)
        #[arg(long, short)]
        token: Option<String>,
    },

    /// Launches the Forge game runtime and prints connection details
    Run {
        /// Path to the scene JSON file to launch
        #[arg(default_value = "templates/hello/scene.json")]
        scene: String,

        /// Run offscreen headless rendering without an interactive window
        #[arg(long, default_value_t = false)]
        headless: bool,

        /// Custom port to request for the bridge
        #[arg(long)]
        port: Option<u16>,
    },

    /// Runs TypeScript type checking on a project's scripts folder
    Check {
        /// Project directory containing scripts/ and tsconfig.json (defaults to current dir)
        #[arg(long, short)]
        project: Option<String>,

        /// Positional project directory argument for backwards compatibility
        #[arg(index = 1)]
        project_pos: Option<String>,

        /// Print diagnostics as structured JSON for autonomous agents
        #[arg(long)]
        json: bool,
    },

    /// Creates a new game project from a template and initializes its procedural assets
    New {
        /// Name of the project directory to create
        name: String,

        /// Starter template to use (defaults to 'platformer')
        #[arg(long, default_value = "platformer")]
        template: String,
    },

    /// Generates .forge/map.json indexing scenes, entities, scripts, assets, clips, and input actions
    Index {
        /// Project directory to index (defaults to current dir)
        #[arg(default_value = ".")]
        project: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Call {
            method,
            params,
            port,
            token,
        } => run_call(method, params, port, token).await,

        Commands::Run {
            scene,
            headless,
            port,
        } => run_runtime(scene, headless, port),

        Commands::Check {
            project,
            project_pos,
            json,
        } => run_check(project.as_deref().or(project_pos.as_deref()), json),

        Commands::New { name, template } => run_new(&name, &template),

        Commands::Index { project } => run_index(&project),
    }
}

fn run_check(
    project_path: Option<&str>,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (success, diagnostics) =
        forge_cli::run_typecheck(project_path).map_err(|e| format!("Typecheck error: {}", e))?;

    if json_output {
        let json_result = serde_json::to_string_pretty(&diagnostics)?;
        println!("{}", json_result);
    } else if diagnostics.is_empty() {
        println!("All scripts passed TypeScript typecheck cleanly.");
    } else {
        for diag in &diagnostics {
            eprintln!(
                "{}({},{}): {} {}: {}",
                diag.file, diag.line, diag.column, diag.severity, diag.code, diag.message
            );
        }
    }

    if !success {
        std::process::exit(1);
    }

    Ok(())
}

fn run_new(name: &str, template: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dest_dir = Path::new(name);
    if dest_dir.exists() {
        return Err(format!("Destination directory '{}' already exists", name).into());
    }

    let template_candidates = [
        format!("templates/{}", template),
        format!("../templates/{}", template),
        format!("../../templates/{}", template),
    ];

    let mut found_template = None;
    for cand in &template_candidates {
        let p = Path::new(cand);
        if p.exists() && p.is_dir() {
            found_template = Some(p.to_path_buf());
            break;
        }
    }

    let src_dir = match found_template {
        Some(p) => p,
        None => {
            return Err(format!("Starter template '{}' not found in templates/", template).into())
        }
    };

    println!(
        "Creating new Forge project '{}' using template '{}'...",
        name, template
    );
    copy_dir_recursive(&src_dir, dest_dir)?;

    // Invoke asset generation if procedural assets need to be generated
    let assets_dir = dest_dir.join("assets");
    fs::create_dir_all(&assets_dir)?;

    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "-p",
        "forge_runtime",
        "--",
        "--scene",
        &dest_dir.join("scene.json").to_string_lossy(),
        "--headless",
        "--frames",
        "1",
    ]);
    let status = cmd.status();
    if let Ok(st) = status {
        if !st.success() {
            tracing::warn!("Asset generation command exited with status: {}", st);
        }
    }

    // Generate .forge/map.json index deterministically
    if let Err(e) = forge_cli::index_project(dest_dir) {
        tracing::warn!("Failed to generate project map index: {}", e);
    }

    println!("Project created successfully at '{}'.", name);
    println!("To run your project:\n  forge run {}/scene.json", name);

    Ok(())
}

fn run_index(project_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let p = Path::new(project_path);
    if !p.exists() {
        return Err(format!("Project directory '{}' does not exist", project_path).into());
    }

    let map_file = forge_cli::index_project(p)?;
    println!(
        "Project index successfully written to '{}'.",
        map_file.display()
    );
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

async fn run_call(
    method: String,
    params_str: String,
    port_arg: Option<u16>,
    token_arg: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (port, token) = resolve_connection_info(port_arg, token_arg)?;

    let params_val: Value = serde_json::from_str(&params_str).unwrap_or_else(|_| json!(params_str));

    let ws_url = format!("ws://127.0.0.1:{}/?token={}", port, token);
    tracing::debug!("Connecting to bridge at {}", ws_url);

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| format!("Failed to connect to bridge at {}: {}", ws_url, e))?;

    let (mut write, mut read) = ws_stream.split();

    let request_id = 1;
    let rpc_request = json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
        "params": params_val
    });

    write
        .send(Message::Text(rpc_request.to_string().into()))
        .await
        .map_err(|e| format!("Failed to send JSON-RPC request: {}", e))?;

    while let Some(msg_result) = read.next().await {
        let msg = msg_result.map_err(|e| format!("WebSocket read error: {}", e))?;
        if let Message::Text(text) = msg {
            let parsed: Value = serde_json::from_str(&text)?;
            if let Some(err) = parsed.get("error") {
                eprintln!("{}", serde_json::to_string_pretty(err)?);
                std::process::exit(1);
            } else if let Some(res) = parsed.get("result") {
                println!("{}", serde_json::to_string_pretty(res)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&parsed)?);
            }
            break;
        }
    }

    Ok(())
}

fn run_runtime(
    scene: String,
    headless: bool,
    port: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "forge_runtime", "--", "--scene", &scene]);

    if headless {
        cmd.arg("--headless");
    }

    if let Some(p) = port {
        cmd.args(["--bridge-port", &p.to_string()]);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());

    let mut child = cmd.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture child stdout")?;
    let mut reader = BufReader::new(stdout);

    let mut line = String::new();
    let mut bridge_ready = false;

    while reader.read_line(&mut line)? > 0 {
        print!("{}", line);
        if line.starts_with("FORGE_BRIDGE_READY ") {
            let json_part = line.trim_start_matches("FORGE_BRIDGE_READY ").trim();
            if let Ok(info) = serde_json::from_str::<BridgeInfo>(json_part) {
                println!("\n=== FORGE BRIDGE READY ===");
                println!("Port: {}", info.port);
                println!("Token: {}", info.token);
                println!("URL: ws://127.0.0.1:{}/?token={}", info.port, info.token);
                println!("==========================\n");

                save_bridge_info(&info);
                bridge_ready = true;
                break;
            }
        }
        line.clear();
    }

    if !bridge_ready {
        tracing::warn!("Runtime exited before emitting FORGE_BRIDGE_READY");
    }

    // Continue streaming stdout
    line.clear();
    while reader.read_line(&mut line)? > 0 {
        print!("{}", line);
        line.clear();
    }

    let status = child.wait()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn resolve_connection_info(
    port_arg: Option<u16>,
    token_arg: Option<String>,
) -> Result<(u16, String), Box<dyn std::error::Error>> {
    if let (Some(port), Some(token)) = (port_arg, token_arg.as_ref()) {
        return Ok((port, token.clone()));
    }

    // Check environment variables
    let env_port = std::env::var("FORGE_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok());
    let env_token = std::env::var("FORGE_TOKEN").ok();

    if let (Some(port), Some(token)) = (port_arg.or(env_port), token_arg.clone().or(env_token)) {
        return Ok((port, token));
    }

    // Check saved bridge.json files
    let paths = [
        Path::new(".forge/bridge.json"),
        Path::new(".forge-bridge.json"),
    ];

    for path in &paths {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(info) = serde_json::from_str::<BridgeInfo>(&content) {
                    return Ok((
                        port_arg.unwrap_or(info.port),
                        token_arg.unwrap_or(info.token),
                    ));
                }
            }
        }
    }

    Err(
        "Connection info not found. Please provide --port <PORT> and --token <TOKEN>, or run runtime via 'forge run <scene>'."
            .into(),
    )
}

fn save_bridge_info(info: &BridgeInfo) {
    let dir = Path::new(".forge");
    let _ = fs::create_dir_all(dir);
    let path = dir.join("bridge.json");
    if let Ok(json_str) = serde_json::to_string_pretty(info) {
        let _ = fs::write(path, json_str);
    }
}
