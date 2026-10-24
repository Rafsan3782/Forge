//! Ring buffer of log and error events for telemetry, bridge notifications, and playtest recording.

use forge_protocol::{GameErrorNotification, GameLogNotification};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A single recorded log or error entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub source_location: Option<String>,
    pub frame: u64,
}

impl LogEntry {
    pub fn to_game_log(&self) -> GameLogNotification {
        GameLogNotification {
            level: self.level.clone(),
            message: self.message.clone(),
            source_location: self.source_location.clone(),
            frame: self.frame,
        }
    }

    pub fn to_game_error(&self) -> GameErrorNotification {
        GameErrorNotification {
            level: "error".to_string(),
            message: self.message.clone(),
            source_location: self.source_location.clone(),
            frame: self.frame,
        }
    }
}

/// Ring buffer holding up to `capacity` entries (default 1000).
#[derive(Debug, Clone)]
pub struct LogRingBuffer {
    capacity: usize,
    entries: Arc<Mutex<VecDeque<LogEntry>>>,
}

impl Default for LogRingBuffer {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl LogRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
        }
    }

    pub fn push(
        &self,
        level: &str,
        message: &str,
        source_location: Option<String>,
        frame: u64,
    ) -> LogEntry {
        let entry = LogEntry {
            level: level.to_string(),
            message: message.to_string(),
            source_location,
            frame,
        };

        let mut lock = self.entries.lock().unwrap();
        if lock.len() >= self.capacity {
            lock.pop_front();
        }
        lock.push_back(entry.clone());
        entry
    }

    pub fn get_entries(&self) -> Vec<LogEntry> {
        let lock = self.entries.lock().unwrap();
        lock.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }

    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_capacity() {
        let buffer = LogRingBuffer::new(3);
        buffer.push("info", "1", None, 1);
        buffer.push("info", "2", None, 2);
        buffer.push("info", "3", None, 3);
        assert_eq!(buffer.len(), 3);

        buffer.push("info", "4", None, 4);
        assert_eq!(buffer.len(), 3);
        let entries = buffer.get_entries();
        assert_eq!(entries[0].message, "2");
        assert_eq!(entries[1].message, "3");
        assert_eq!(entries[2].message, "4");
    }
}
