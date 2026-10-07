//! Time tracking resource for game update systems and render interpolation.

use crate::ecs::Resource;
use std::time::Duration;

/// Time resource containing frame timing, fixed timestep information, and render interpolation alpha.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Time {
    /// Delta duration of the current frame.
    pub delta: Duration,
    /// Delta time in seconds as an `f32`.
    pub delta_seconds: f32,
    /// Total elapsed running time.
    pub elapsed: Duration,
    /// Total elapsed time in seconds as an `f64`.
    pub elapsed_seconds: f64,
    /// Duration of a single fixed simulation tick (default 1/60th second).
    pub fixed_delta: Duration,
    /// Duration of a fixed simulation tick in seconds as an `f32`.
    pub fixed_delta_seconds: f32,
    /// Interpolation factor between the previous and current physics/simulation tick in `[0.0, 1.0)`.
    pub alpha: f32,
    /// Total count of fixed updates executed since startup.
    pub tick_count: u64,
}

impl Time {
    /// Default fixed timestep frequency (60 Hz).
    pub const DEFAULT_FIXED_HZ: f64 = 60.0;

    /// Creates a new `Time` resource with the specified fixed update frequency in Hz.
    pub fn new(fixed_hz: f64) -> Self {
        let fixed_hz_clamped = if fixed_hz > 0.0 {
            fixed_hz
        } else {
            Self::DEFAULT_FIXED_HZ
        };
        let fixed_delta_secs = 1.0 / fixed_hz_clamped;
        let fixed_delta = Duration::from_secs_f64(fixed_delta_secs);

        Self {
            delta: Duration::ZERO,
            delta_seconds: 0.0,
            elapsed: Duration::ZERO,
            elapsed_seconds: 0.0,
            fixed_delta,
            fixed_delta_seconds: fixed_delta_secs as f32,
            alpha: 0.0,
            tick_count: 0,
        }
    }

    /// Advances the frame timer by the specified frame duration.
    pub fn advance_frame(&mut self, frame_delta: Duration) {
        self.delta = frame_delta;
        self.delta_seconds = frame_delta.as_secs_f32();
        self.elapsed += frame_delta;
        self.elapsed_seconds = self.elapsed.as_secs_f64();
    }

    /// Advances the fixed simulation tick counter.
    pub fn advance_fixed_tick(&mut self) {
        self.tick_count += 1;
    }

    /// Sets the render interpolation factor `alpha` in `[0.0, 1.0]`.
    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha.clamp(0.0, 1.0);
    }

    /// Returns the target frequency of fixed timestep updates in Hertz.
    pub fn fixed_hz(&self) -> f64 {
        1.0 / self.fixed_delta.as_secs_f64()
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new(Self::DEFAULT_FIXED_HZ)
    }
}
