//! File watcher and hot-reload debouncer using `notify`.
//!
//! Watches project files (textures, sprite sheets, animation clips, scenes)
//! and dispatches reloads into the running engine.

use crate::error::AssetError;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

/// Change type detected by the hot reload watcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotReloadEvent {
    /// Texture file changed on disk.
    Texture(PathBuf),
    /// SpriteSheet layout file changed on disk.
    SpriteSheet(PathBuf),
    /// Animation clip file changed on disk.
    Animation(PathBuf),
    /// Scene file changed on disk.
    Scene(PathBuf),
    /// Asset metadata `.meta.json` file changed on disk.
    Meta(PathBuf),
    /// Other generic asset changed.
    Other(PathBuf),
}

impl HotReloadEvent {
    /// Classifies an event based on file path.
    pub fn from_path(path: PathBuf) -> Self {
        let path_str = path.to_string_lossy();
        if path_str.ends_with(".meta.json") {
            HotReloadEvent::Meta(path)
        } else if path_str.ends_with(".png")
            || path_str.ends_with(".jpg")
            || path_str.ends_with(".jpeg")
        {
            HotReloadEvent::Texture(path)
        } else if path_str.ends_with(".spritesheet.json") || path_str.ends_with(".atlas.json") {
            HotReloadEvent::SpriteSheet(path)
        } else if path_str.ends_with(".anim.json") || path_str.ends_with(".animation.json") {
            HotReloadEvent::Animation(path)
        } else if path_str.ends_with(".scene.json") || path_str.ends_with("scene.json") {
            HotReloadEvent::Scene(path)
        } else {
            HotReloadEvent::Other(path)
        }
    }

    /// Returns the target path.
    pub fn path(&self) -> &Path {
        match self {
            HotReloadEvent::Texture(p)
            | HotReloadEvent::SpriteSheet(p)
            | HotReloadEvent::Animation(p)
            | HotReloadEvent::Scene(p)
            | HotReloadEvent::Meta(p)
            | HotReloadEvent::Other(p) => p.as_path(),
        }
    }
}

/// Hot reload watcher with debounced event queue.
pub struct HotReloadWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
    pending_paths: HashSet<PathBuf>,
    last_event_time: Option<Instant>,
    debounce_duration: Duration,
}

impl HotReloadWatcher {
    /// Default debounce delay (e.g. 50ms to accumulate rapid multi-write events from editors).
    pub const DEFAULT_DEBOUNCE_MS: u64 = 50;

    /// Starts a background watcher on the specified path or directory.
    pub fn new(watch_path: impl AsRef<Path>) -> Result<Self, AssetError> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;

        if watch_path.as_ref().exists() {
            watcher
                .watch(watch_path.as_ref(), RecursiveMode::Recursive)
                .map_err(|e| std::io::Error::other(e.to_string()))?;
        }

        Ok(Self {
            _watcher: watcher,
            rx,
            pending_paths: HashSet::new(),
            last_event_time: None,
            debounce_duration: Duration::from_millis(Self::DEFAULT_DEBOUNCE_MS),
        })
    }

    /// Sets custom debounce duration.
    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }

    /// Polls for debounced file modification events.
    ///
    /// Drains incoming raw events and returns classified events once the debounce
    /// interval has elapsed since the last write event.
    pub fn poll_events(&mut self) -> Vec<HotReloadEvent> {
        while let Ok(res) = self.rx.try_recv() {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        for path in event.paths {
                            if path.is_file() {
                                self.pending_paths.insert(path);
                                self.last_event_time = Some(Instant::now());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(last_time) = self.last_event_time {
            if last_time.elapsed() >= self.debounce_duration && !self.pending_paths.is_empty() {
                self.last_event_time = None;
                let drained: Vec<HotReloadEvent> = self
                    .pending_paths
                    .drain()
                    .map(HotReloadEvent::from_path)
                    .collect();
                return drained;
            }
        }

        Vec::new()
    }
}
