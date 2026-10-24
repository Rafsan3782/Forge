//! Headless playtesting, test harnesses, and telemetry collection.

use forge_script::{LogEntry, LogRingBuffer};
use std::sync::Arc;

/// Playtest session recorder capturing logs, errors, and performance.
pub struct PlaytestRecorder {
    pub ring_buffer: Arc<LogRingBuffer>,
}

impl Default for PlaytestRecorder {
    fn default() -> Self {
        Self::new(Arc::new(LogRingBuffer::new(1000)))
    }
}

impl PlaytestRecorder {
    pub fn new(ring_buffer: Arc<LogRingBuffer>) -> Self {
        Self { ring_buffer }
    }

    pub fn snapshot_logs(&self) -> Vec<LogEntry> {
        self.ring_buffer.get_entries()
    }
}
