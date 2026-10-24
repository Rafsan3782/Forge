//! Forge Script: QuickJS-NG embedded runtime with SWC TypeScript transpilation.

#![allow(clippy::result_large_err)]
#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::type_complexity)]
#![allow(clippy::field_reassign_with_default)]

pub mod bindings;
pub mod error;
pub mod runtime;
pub mod sourcemap;
pub mod systems;
pub mod transpiler;

pub use bindings::{bind_forge_api, ScriptCommand, ScriptExecutionContext};
pub use error::ScriptExecutionError;
pub use runtime::{LoadedScript, SandboxConfig, ScriptRuntime};
pub use sourcemap::{ScriptSourceMapper, SourceLocation};
pub use systems::{
    script_apply_commands_system, script_fixed_update_system, script_prepare_frame_system,
    script_start_system, script_update_system, ScriptServer,
};
pub use transpiler::{transpile_typescript, TranspiledScript};
