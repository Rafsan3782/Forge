//! # Forge Audio
//!
//! Audio system for Forge built on the Kira audio library.
//!
//! Features:
//! - Multi-format asset playback (`.wav`, `.ogg`, `.mp3`) through asset database
//! - Audio buses (`master`, `music`, `sfx`) with linear volume control
//! - Fade in / fade out and crossfading for music
//! - `AudioSource` component with 2D spatial pan and distance falloff
//! - `AudioListener` component for camera tracking
//! - Headless / CI fallback to a null backend when no audio output device is present
//! - Non-panicking error handling for missing or corrupt sound assets

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod server;

pub use server::{
    audio_source_start_system, audio_spatial_update_system, ActiveSound, AudioServer,
    BackendManager,
};

// Re-export audio components and buses from forge_core
pub use forge_core::components::{AudioBus, AudioListener, AudioSource};

#[cfg(test)]
mod tests;
