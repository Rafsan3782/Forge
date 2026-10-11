//! AnimationClip asset data model, events, and playback timing.
//!
//! An `AnimationClip` specifies:
//! - An ordered list of frame indices
//! - Frames per second (`fps`)
//! - Loop mode (`Loop`, `Once`, `PingPong`)
//! - Animation events triggered at specific frame indices

use crate::error::AssetError;
use serde::{Deserialize, Serialize};

/// Playback looping mode for an animation clip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopMode {
    /// Repeats indefinitely from the start when reaching the end.
    Loop,
    /// Plays once and freezes on the final frame.
    Once,
    /// Alternates direction between forward and backward.
    PingPong,
}

impl Default for LoopMode {
    fn default() -> Self {
        Self::Loop
    }
}

/// An event triggered when an animation playback reaches a specific frame index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationEvent {
    /// Frame index at which this event fires.
    pub frame: usize,
    /// Name or identifier of the event (e.g. "footstep", "attack_hit").
    pub name: String,
    /// Optional event payload parameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
}

/// An animation clip asset defining frame sequence, playback rate, loop mode, and events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationClip {
    /// Human-readable name of the clip.
    pub name: String,
    /// Target sprite sheet asset ID (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spritesheet_asset_id: Option<String>,
    /// Sequence of frame indices to display.
    pub frames: Vec<u32>,
    /// Frame playback rate in frames per second.
    pub fps: f32,
    /// Loop behavior.
    #[serde(default)]
    pub loop_mode: LoopMode,
    /// Events triggered at particular frame steps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<AnimationEvent>,
}

impl AnimationClip {
    /// Creates a new animation clip.
    pub fn new(name: impl Into<String>, frames: Vec<u32>, fps: f32) -> Self {
        Self {
            name: name.into(),
            spritesheet_asset_id: None,
            frames,
            fps: fps.max(0.1),
            loop_mode: LoopMode::Loop,
            events: Vec::new(),
        }
    }

    /// Sets the loop mode.
    pub fn with_loop_mode(mut self, loop_mode: LoopMode) -> Self {
        self.loop_mode = loop_mode;
        self
    }

    /// Adds an event trigger.
    pub fn with_event(
        mut self,
        frame: usize,
        name: impl Into<String>,
        payload: Option<String>,
    ) -> Self {
        self.events.push(AnimationEvent {
            frame,
            name: name.into(),
            payload,
        });
        self
    }

    /// Total duration of the animation clip in seconds.
    pub fn duration_seconds(&self) -> f32 {
        if self.frames.is_empty() || self.fps <= 0.0 {
            0.0
        } else {
            self.frames.len() as f32 / self.fps
        }
    }

    /// Computes the active frame index and triggered events given an elapsed time.
    pub fn sample(&self, elapsed_seconds: f32) -> (usize, u32) {
        if self.frames.is_empty() {
            return (0, 0);
        }

        let total_frames = self.frames.len();
        if total_frames == 1 {
            return (0, self.frames[0]);
        }

        let total_duration = self.duration_seconds();
        let frame_duration = 1.0 / self.fps;

        let index = match self.loop_mode {
            LoopMode::Once => {
                let frame_idx = (elapsed_seconds / frame_duration).floor() as usize;
                frame_idx.min(total_frames - 1)
            }
            LoopMode::Loop => {
                let wrapped_time = if total_duration > 0.0 {
                    elapsed_seconds.rem_euclid(total_duration)
                } else {
                    0.0
                };
                let frame_idx = (wrapped_time / frame_duration).floor() as usize;
                frame_idx.min(total_frames - 1)
            }
            LoopMode::PingPong => {
                let cycle_duration = total_duration * 2.0;
                let wrapped_time = if cycle_duration > 0.0 {
                    elapsed_seconds.rem_euclid(cycle_duration)
                } else {
                    0.0
                };
                if wrapped_time <= total_duration {
                    let frame_idx = (wrapped_time / frame_duration).floor() as usize;
                    frame_idx.min(total_frames - 1)
                } else {
                    let backward_time = wrapped_time - total_duration;
                    let frame_idx = (backward_time / frame_duration).floor() as usize;
                    (total_frames - 1).saturating_sub(frame_idx)
                }
            }
        };

        (index, self.frames[index])
    }

    /// Loads an AnimationClip from a JSON string.
    pub fn from_json_str(json: &str) -> Result<Self, AssetError> {
        let clip = serde_json::from_str(json)?;
        Ok(clip)
    }

    /// Serializes an AnimationClip to a JSON string.
    pub fn to_json_string(&self) -> Result<String, AssetError> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }
}
