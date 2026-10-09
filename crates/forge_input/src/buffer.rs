//! Input event recording into a bounded ring buffer for playtest telemetry.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A single recorded input event stamped with its simulation frame number.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedInputEvent {
    /// Frame number at which the event was recorded.
    pub frame: u64,
    /// Type/name of the input event (e.g. "key_down", "action_pressed", "mouse_move").
    pub event_type: String,
    /// Target input key, button, or action name.
    pub target: String,
    /// Value associated with the event (e.g. 1.0 for button press, axis float, etc.).
    pub value: f32,
}

/// Ring buffer resource storing recent input events.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct InputRingBuffer {
    events: VecDeque<RecordedInputEvent>,
    capacity: usize,
}

impl InputRingBuffer {
    /// Default buffer capacity (e.g. 4096 events).
    pub const DEFAULT_CAPACITY: usize = 4096;

    /// Creates a new ring buffer with the given maximum capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity.min(4096)),
            capacity,
        }
    }

    /// Records a new event into the buffer, discarding oldest entries if capacity is reached.
    pub fn record(
        &mut self,
        frame: u64,
        event_type: impl Into<String>,
        target: impl Into<String>,
        value: f32,
    ) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(RecordedInputEvent {
            frame,
            event_type: event_type.into(),
            target: target.into(),
            value,
        });
    }

    /// Returns the number of recorded events currently in the buffer.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns whether the ring buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Returns a slice/iterator over recorded events.
    pub fn events(&self) -> &VecDeque<RecordedInputEvent> {
        &self.events
    }

    /// Clears all recorded events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl Default for InputRingBuffer {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}
