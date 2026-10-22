//! Physics character controller component and kinematic helper.

use forge_core::ecs::Component;
use forge_core::Vec2;
use serde::{Deserialize, Serialize};

/// Character controller component providing kinematic move-and-slide movement.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterController2D {
    /// Desired movement translation for the current fixed timestep update.
    #[serde(default)]
    pub translation: Vec2,
    /// Offset for auto-stepping up stairs or ledges.
    #[serde(default = "default_autostep_height")]
    pub autostep_max_height: f32,
    /// Minimum autostep width.
    #[serde(default = "default_autostep_width")]
    pub autostep_min_width: f32,
    /// Maximum slope angle in radians (e.g. 45 degrees = ~0.785 rad).
    #[serde(default = "default_max_slope_angle")]
    pub max_slope_angle: f32,
    /// Minimum slope angle for sliding down in radians.
    #[serde(default = "default_min_slope_slide_angle")]
    pub min_slope_slide_angle: f32,
    /// Skin thickness / contact offset to avoid numerical tunneling.
    #[serde(default = "default_offset")]
    pub offset: f32,
    /// Whether the controller is currently grounded (standing on a surface).
    #[serde(default)]
    pub is_grounded: bool,
    /// Whether the character is sliding down a steep slope.
    #[serde(default)]
    pub is_sliding_down_slope: bool,
    /// Up vector defining "ground" direction (typically `[0.0, 1.0]`).
    #[serde(default = "default_up")]
    pub up: Vec2,
    /// Whether to slide along surfaces instead of stopping abruptly.
    #[serde(default = "default_slide")]
    pub slide_enabled: bool,
    /// One-way platform filter mask. If non-zero, surfaces with this layer are ignored when moving upward.
    #[serde(default)]
    pub one_way_platform_mask: u32,
}

fn default_autostep_height() -> f32 {
    8.0
}

fn default_autostep_width() -> f32 {
    4.0
}

fn default_max_slope_angle() -> f32 {
    std::f32::consts::FRAC_PI_4 // 45 degrees
}

fn default_min_slope_slide_angle() -> f32 {
    std::f32::consts::FRAC_PI_4 // 45 degrees
}

fn default_offset() -> f32 {
    0.05
}

fn default_up() -> Vec2 {
    Vec2::new(0.0, 1.0)
}

fn default_slide() -> bool {
    true
}

impl Default for CharacterController2D {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            autostep_max_height: 8.0,
            autostep_min_width: 4.0,
            max_slope_angle: std::f32::consts::FRAC_PI_4,
            min_slope_slide_angle: std::f32::consts::FRAC_PI_4,
            offset: 0.05,
            is_grounded: false,
            is_sliding_down_slope: false,
            up: Vec2::new(0.0, 1.0),
            slide_enabled: true,
            one_way_platform_mask: 0,
        }
    }
}

impl CharacterController2D {
    /// Creates a character controller with standard platformer defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the desired translation for the current frame/tick.
    pub fn with_translation(mut self, translation: Vec2) -> Self {
        self.translation = translation;
        self
    }

    /// Sets maximum step height.
    pub fn with_autostep(mut self, max_height: f32, min_width: f32) -> Self {
        self.autostep_max_height = max_height;
        self.autostep_min_width = min_width;
        self
    }

    /// Sets maximum walkable slope angle in radians.
    pub fn with_max_slope_angle(mut self, angle: f32) -> Self {
        self.max_slope_angle = angle;
        self
    }
}
