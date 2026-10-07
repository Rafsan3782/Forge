//! Game loop execution driving fixed timestep simulation and schedule stages.

use crate::ecs::{IntoSystemConfigs, Schedule, World};
use crate::time::Time;
use std::time::Duration;

/// Simple schedule stages for fixed-timestep game updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    /// Systems executed before the main simulation update (input sampling, setup).
    PreUpdate,
    /// Main simulation systems (physics, gameplay logic, AI).
    Update,
    /// Systems executed after main update (cleanup, transform propagation, state sync).
    PostUpdate,
}

/// Fixed-timestep game loop runner that advances schedules and computes render interpolation factor.
pub struct GameLoop {
    /// Schedule executed during PreUpdate stage.
    pub pre_update: Schedule,
    /// Schedule executed during Update stage.
    pub update: Schedule,
    /// Schedule executed during PostUpdate stage.
    pub post_update: Schedule,
    /// Accumulated delta time waiting to be consumed by fixed simulation ticks.
    accumulator: Duration,
    /// Maximum number of fixed updates per step to prevent spiral of death on lag spikes.
    max_sub_steps: u32,
}

impl GameLoop {
    /// Default maximum sub-steps to process per frame.
    pub const DEFAULT_MAX_SUB_STEPS: u32 = 10;

    /// Creates a new game loop with empty PreUpdate, Update, and PostUpdate schedules.
    pub fn new() -> Self {
        Self {
            pre_update: Schedule::default(),
            update: Schedule::default(),
            post_update: Schedule::default(),
            accumulator: Duration::ZERO,
            max_sub_steps: Self::DEFAULT_MAX_SUB_STEPS,
        }
    }

    /// Sets the maximum fixed sub-steps executed per frame.
    pub fn with_max_sub_steps(mut self, max_sub_steps: u32) -> Self {
        self.max_sub_steps = max_sub_steps.max(1);
        self
    }

    /// Adds a system to the PreUpdate schedule.
    pub fn add_pre_update_system<M>(&mut self, system: impl IntoSystemConfigs<M>) -> &mut Self {
        self.pre_update.add_systems(system);
        self
    }

    /// Adds a system to the main Update schedule.
    pub fn add_update_system<M>(&mut self, system: impl IntoSystemConfigs<M>) -> &mut Self {
        self.update.add_systems(system);
        self
    }

    /// Adds a system to the PostUpdate schedule.
    pub fn add_post_update_system<M>(&mut self, system: impl IntoSystemConfigs<M>) -> &mut Self {
        self.post_update.add_systems(system);
        self
    }

    /// Executes one frame step with the given frame delta duration.
    ///
    /// Consumes fixed delta increments running PreUpdate, Update, and PostUpdate schedules,
    /// then updates the `Time` resource with the remaining interpolation `alpha`.
    pub fn step(&mut self, world: &mut World, frame_delta: Duration) {
        // Ensure Time resource exists in the world
        if !world.contains_resource::<Time>() {
            world.insert_resource(Time::default());
        }

        let fixed_delta = {
            let mut time = world.resource_mut::<Time>();
            time.advance_frame(frame_delta);
            time.fixed_delta
        };

        self.accumulator += frame_delta;
        let mut steps = 0;

        while self.accumulator >= fixed_delta && steps < self.max_sub_steps {
            self.accumulator -= fixed_delta;
            steps += 1;

            {
                let mut time = world.resource_mut::<Time>();
                time.advance_fixed_tick();
            }

            self.pre_update.run(world);
            self.update.run(world);
            self.post_update.run(world);
        }

        // Clamp accumulator if we reached the maximum sub-step limit to prevent spiral of death
        if steps >= self.max_sub_steps {
            self.accumulator = Duration::ZERO;
        }

        let alpha = if fixed_delta.as_secs_f64() > 0.0 {
            (self.accumulator.as_secs_f64() / fixed_delta.as_secs_f64()) as f32
        } else {
            0.0
        };

        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.set_alpha(alpha);
        }
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}
