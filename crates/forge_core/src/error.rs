//! Error types for the Forge core crate.

use thiserror::Error;

/// Errors that can occur during scene operations and serialization.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SceneError {
    /// Entity with the specified ID was not found.
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    /// Entity ID is already taken.
    #[error("Duplicate entity ID: {0}")]
    DuplicateEntityId(String),

    /// Component deserialization or data parsing failed.
    #[error("Failed to parse component '{component_name}': {message}")]
    ComponentParseError {
        /// Name of the component.
        component_name: String,
        /// Detail message of the failure.
        message: String,
    },

    /// The specified component was not found on the entity.
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// The specified component name is unrecognized.
    #[error("Unknown component: {0}")]
    UnknownComponent(String),

    /// Failed to read or parse JSON data.
    #[error("JSON error: {0}")]
    JsonError(String),

    /// File I/O failure.
    #[error("I/O error: {0}")]
    IoError(String),

    /// Circular or invalid entity hierarchy.
    #[error("Invalid entity hierarchy: {0}")]
    InvalidHierarchy(String),
}

impl From<serde_json::Error> for SceneError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError(err.to_string())
    }
}

impl From<std::io::Error> for SceneError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

/// Generic errors originating from Forge core systems.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// A scene operation failed.
    #[error("Scene error: {0}")]
    Scene(#[from] SceneError),

    /// Internal error message.
    #[error("Core error: {0}")]
    Message(String),
}
