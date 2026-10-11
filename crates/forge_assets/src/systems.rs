//! Asset management systems, stores, and fixed-timestep animation systems.

use crate::animation::AnimationClip;
use crate::error::AssetError;
use crate::spritesheet::SpriteSheet;
use forge_core::{Animator, EventBus, Query, Res, ResMut, Resource, Sprite, Time};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Storage and caching for loaded `AnimationClip` assets.
#[derive(Resource, Debug, Default, Clone)]
pub struct AnimationStore {
    clips: HashMap<String, AnimationClip>,
    base_dir: Option<PathBuf>,
}

impl AnimationStore {
    /// Creates a new empty animation store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base directory for loading clips.
    pub fn set_base_dir(&mut self, dir: impl Into<PathBuf>) {
        self.base_dir = Some(dir.into());
    }

    /// Registers an animation clip under an asset ID.
    pub fn insert(&mut self, asset_id: impl Into<String>, clip: AnimationClip) {
        self.clips.insert(asset_id.into(), clip);
    }

    /// Retrieves a reference to an animation clip by asset ID.
    pub fn get(&self, asset_id: &str) -> Option<&AnimationClip> {
        self.clips.get(asset_id)
    }

    /// Loads or fetches an animation clip from file or cache.
    pub fn get_or_load(&mut self, asset_id: &str) -> Result<&AnimationClip, AssetError> {
        if !self.clips.contains_key(asset_id) {
            let mut candidate_paths = Vec::new();
            if let Some(base) = &self.base_dir {
                candidate_paths.push(base.join(asset_id));
            }
            candidate_paths.push(PathBuf::from(asset_id));

            let mut resolved = None;
            for p in &candidate_paths {
                if p.exists() && p.is_file() {
                    resolved = Some(p.clone());
                    break;
                }
            }

            let path = resolved.ok_or_else(|| {
                AssetError::AssetNotFound(format!("AnimationClip asset '{}' not found", asset_id))
            })?;

            let content = fs::read_to_string(path)?;
            let clip = AnimationClip::from_json_str(&content)?;
            self.clips.insert(asset_id.to_string(), clip);
        }

        Ok(self.clips.get(asset_id).unwrap())
    }

    /// Reloads an animation clip from disk.
    pub fn reload(&mut self, asset_id: &str, path: impl AsRef<Path>) -> Result<(), AssetError> {
        let content = fs::read_to_string(path)?;
        let clip = AnimationClip::from_json_str(&content)?;
        self.clips.insert(asset_id.to_string(), clip);
        Ok(())
    }
}

/// Storage and caching for loaded `SpriteSheet` assets.
#[derive(Resource, Debug, Default, Clone)]
pub struct SpriteSheetStore {
    sheets: HashMap<String, SpriteSheet>,
    base_dir: Option<PathBuf>,
}

impl SpriteSheetStore {
    /// Creates a new empty sprite sheet store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base directory for loading sheets.
    pub fn set_base_dir(&mut self, dir: impl Into<PathBuf>) {
        self.base_dir = Some(dir.into());
    }

    /// Registers a sprite sheet under an asset ID.
    pub fn insert(&mut self, asset_id: impl Into<String>, sheet: SpriteSheet) {
        self.sheets.insert(asset_id.into(), sheet);
    }

    /// Retrieves a sprite sheet by asset ID.
    pub fn get(&self, asset_id: &str) -> Option<&SpriteSheet> {
        self.sheets.get(asset_id)
    }

    /// Loads or fetches a sprite sheet from file or cache.
    pub fn get_or_load(&mut self, asset_id: &str) -> Result<&SpriteSheet, AssetError> {
        if !self.sheets.contains_key(asset_id) {
            let mut candidate_paths = Vec::new();
            if let Some(base) = &self.base_dir {
                candidate_paths.push(base.join(asset_id));
            }
            candidate_paths.push(PathBuf::from(asset_id));

            let mut resolved = None;
            for p in &candidate_paths {
                if p.exists() && p.is_file() {
                    resolved = Some(p.clone());
                    break;
                }
            }

            let path = resolved.ok_or_else(|| {
                AssetError::AssetNotFound(format!("SpriteSheet asset '{}' not found", asset_id))
            })?;

            let content = fs::read_to_string(path)?;
            let sheet = SpriteSheet::from_json_str(&content)?;
            self.sheets.insert(asset_id.to_string(), sheet);
        }

        Ok(self.sheets.get(asset_id).unwrap())
    }

    /// Reloads a sprite sheet from disk.
    pub fn reload(&mut self, asset_id: &str, path: impl AsRef<Path>) -> Result<(), AssetError> {
        let content = fs::read_to_string(path)?;
        let sheet = SpriteSheet::from_json_str(&content)?;
        self.sheets.insert(asset_id.to_string(), sheet);
        Ok(())
    }
}

/// Fixed timestep animation system advancing `Animator` components and setting `Sprite.frame`.
/// Emits any triggered `AnimationEvent` items to the `EventBus` if present.
pub fn animator_update_system(
    time: Res<Time>,
    mut anim_store: ResMut<AnimationStore>,
    mut event_bus: Option<ResMut<EventBus>>,
    mut query: Query<(&mut Animator, &mut Sprite)>,
) {
    let dt = time.fixed_delta_seconds;

    for (mut animator, mut sprite) in query.iter_mut() {
        if !animator.playing || animator.speed == 0.0 {
            continue;
        }

        let clip_asset_id = animator.clip_asset_id.clone();
        if clip_asset_id.is_empty() {
            continue;
        }

        // Try to get or load the clip
        let clip = match anim_store.get_or_load(&clip_asset_id) {
            Ok(c) => c.clone(),
            Err(_) => continue,
        };

        let prev_frame_idx = animator.current_frame_index;
        animator.elapsed_seconds += dt * animator.speed;
        if animator.elapsed_seconds < 0.0 {
            animator.elapsed_seconds = 0.0;
        }

        let (new_frame_idx, frame_number) = clip.sample(animator.elapsed_seconds);
        animator.current_frame_index = new_frame_idx;
        sprite.frame = Some(frame_number);

        // Check if new frame was entered, emit events
        if new_frame_idx != prev_frame_idx {
            if let Some(ref mut bus) = event_bus {
                for ev in &clip.events {
                    if ev.frame == new_frame_idx {
                        bus.send(ev.clone());
                    }
                }
            }
        }
    }
}
