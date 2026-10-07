//! Error definitions for the asset pipeline.

use forge_render::RenderError;
use thiserror::Error;

/// Errors that can occur during asset loading, decoding, or GPU uploading.
#[derive(Debug, Error)]
pub enum AssetError {
    /// File I/O failure.
    #[error("Asset I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Image decoding failure.
    #[error("Image decoding error: {0}")]
    Image(#[from] image::ImageError),

    /// GPU texture creation failure.
    #[error("Render GPU error: {0}")]
    Render(#[from] RenderError),

    /// Missing or unrecognized asset format.
    #[error("Asset '{0}' was not found or has an unsupported format")]
    AssetNotFound(String),
}
