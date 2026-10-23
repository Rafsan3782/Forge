//! Script execution errors and structured reporting.

use forge_protocol::ScriptErrorEvent;

/// Structured error returned from script execution.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptExecutionError {
    pub entity_id: String,
    pub script_path: String,
    pub message: String,
    pub stack: Option<String>,
    pub file: Option<String>,
    pub line: Option<i64>,
    pub column: Option<i64>,
}

impl ScriptExecutionError {
    /// Converts to a protocol `ScriptErrorEvent`.
    pub fn to_event(&self) -> ScriptErrorEvent {
        ScriptErrorEvent {
            entity_id: self.entity_id.clone(),
            script_path: self.script_path.clone(),
            message: self.message.clone(),
            stack: self.stack.clone(),
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        }
    }
}
