//! Error types for the rendering subsystem.

use thiserror::Error;

/// Errors that can occur during rendering or GPU operations.
#[derive(Debug, Error)]
pub enum RenderError {
    /// No compatible GPU adapter was found.
    #[error("No compatible GPU adapter found (hardware or software fallback)")]
    NoAdapterFound,

    /// Requesting a GPU device or queue failed.
    #[error("Failed to request GPU device: {0}")]
    RequestDeviceFailed(String),

    /// Surface configuration failed.
    #[error("Failed to configure surface: {0}")]
    SurfaceConfigFailed(String),

    /// Acquiring the next surface texture failed.
    #[error("Failed to acquire next surface texture: {0}")]
    SurfaceTextureError(String),

    /// Buffer mapping or reading failed.
    #[error("Failed to map or read GPU buffer: {0}")]
    BufferReadError(String),

    /// Image dimensions or buffer size is invalid.
    #[error("Invalid image dimensions ({width}x{height}) or size")]
    InvalidDimensions {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
    },
}
