//! High-level audio backend manager wrapping Kira.
//!
//! Handles playback routing across `master`, `music`, and `sfx` buses,
//! seamless fallback to a null / mock backend when no audio output device exists (e.g. in CI),
//! and graceful error handling on corrupt or missing audio files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use forge_core::components::{AudioBus, AudioListener, AudioSource, Transform2D, Vec2};
use forge_core::ecs::{Commands, Entity, Query, ResMut, Resource};
use kira::backend::mock::MockBackend;
use kira::backend::DefaultBackend;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::sound::PlaybackState;
use kira::{AudioManager, AudioManagerSettings, Decibels, Easing, PlaybackRate, StartTime, Tween};

/// Backend engine enum abstracting hardware CPAL backend vs headless mock backend.
pub enum BackendManager {
    /// Hardware CPAL backend for audio output device.
    Cpal(Box<AudioManager<DefaultBackend>>),
    /// Headless mock backend for CI or systems with no audio output device.
    Mock(Box<AudioManager<MockBackend>>),
}

/// Active sound playback instance tracking Kira handle and bus.
pub struct ActiveSound {
    /// Handle to active Kira static sound.
    pub handle: StaticSoundHandle,
    /// Bus channel this sound is assigned to.
    pub bus: AudioBus,
    /// Target volume multiplier (linear 0.0 to 1.0+).
    pub target_volume: f32,
    /// Optional Entity if this sound is bound to an AudioSource component.
    pub entity: Option<Entity>,
}

/// Central audio manager resource storing sound cache, active handles, and bus volumes.
#[derive(Resource)]
pub struct AudioServer {
    /// Active Kira audio manager (hardware or mock).
    backend: BackendManager,
    /// Master bus linear volume (0.0 - 1.0).
    pub master_volume: f32,
    /// Music bus linear volume (0.0 - 1.0).
    pub music_volume: f32,
    /// Sfx bus linear volume (0.0 - 1.0).
    pub sfx_volume: f32,
    /// Cached static sound data by canonical path or asset ID.
    sound_cache: HashMap<String, StaticSoundData>,
    /// Active sound instances by ID.
    active_sounds: HashMap<String, ActiveSound>,
    /// Counter for generating unique playback IDs.
    id_counter: AtomicU64,
    /// Whether the server is running on a null / mock backend.
    pub is_null_backend: bool,
    /// Project assets directory root.
    assets_dir: PathBuf,
}

/// Helper converting linear amplitude [0.0, 1.0+] to Decibels for Kira.
pub fn linear_to_decibels(linear: f32) -> Decibels {
    if linear <= 0.0001 {
        Decibels::SILENCE
    } else if (linear - 1.0).abs() < 1e-4 {
        Decibels::IDENTITY
    } else {
        Decibels(20.0 * linear.log10())
    }
}

impl AudioServer {
    /// Initializes a new audio server, attempting to open default audio hardware via CPAL.
    ///
    /// If no audio output device is found (e.g. in CI or headless environments),
    /// falls back to a null `MockBackend` and logs one clear warning without failing.
    pub fn new(assets_dir: impl AsRef<Path>) -> Self {
        let assets_dir = assets_dir.as_ref().to_path_buf();
        let mut is_null_backend = false;

        let backend = match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
            Ok(cpal_mgr) => BackendManager::Cpal(Box::new(cpal_mgr)),
            Err(err) => {
                tracing::warn!(
                    "No audio device detected or failed to initialize CPAL audio device ({}). Falling back to null audio backend.",
                    err
                );
                is_null_backend = true;
                let mock_mgr = AudioManager::<MockBackend>::new(AudioManagerSettings::default())
                    .expect("Failed to initialize MockBackend audio manager");
                BackendManager::Mock(Box::new(mock_mgr))
            }
        };

        Self {
            backend,
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            sound_cache: HashMap::new(),
            active_sounds: HashMap::new(),
            id_counter: AtomicU64::new(1),
            is_null_backend,
            assets_dir,
        }
    }

    /// Explicitly creates an audio server with null / mock backend (ideal for deterministic tests).
    pub fn new_null(assets_dir: impl AsRef<Path>) -> Self {
        let mock_mgr = AudioManager::<MockBackend>::new(AudioManagerSettings::default())
            .expect("Failed to initialize MockBackend audio manager");

        Self {
            backend: BackendManager::Mock(Box::new(mock_mgr)),
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            sound_cache: HashMap::new(),
            active_sounds: HashMap::new(),
            id_counter: AtomicU64::new(1),
            is_null_backend: true,
            assets_dir: assets_dir.as_ref().to_path_buf(),
        }
    }

    /// Sets the base assets directory.
    pub fn set_assets_dir(&mut self, dir: impl AsRef<Path>) {
        self.assets_dir = dir.as_ref().to_path_buf();
    }

    /// Resolves an asset identifier or path to an absolute or project-relative file path.
    pub fn resolve_path(&self, asset_ref: &str) -> PathBuf {
        let p = Path::new(asset_ref);
        if p.exists() {
            p.to_path_buf()
        } else {
            let candidate = self.assets_dir.join(asset_ref);
            if candidate.exists() {
                candidate
            } else {
                p.to_path_buf()
            }
        }
    }

    /// Clears cached sound data for hot reload.
    pub fn invalidate_cache(&mut self, file_path: &Path) {
        let key = file_path.to_string_lossy().to_string();
        self.sound_cache.remove(&key);
        if let Some(stem) = file_path.file_name().and_then(|f| f.to_str()) {
            self.sound_cache.remove(stem);
        }
    }

    /// Loads or returns cached `StaticSoundData` without panicking on corrupt or missing files.
    pub fn get_or_load_sound(&mut self, asset_ref: &str) -> Result<StaticSoundData, String> {
        if let Some(cached) = self.sound_cache.get(asset_ref) {
            return Ok(cached.clone());
        }

        let resolved = self.resolve_path(asset_ref);
        if !resolved.exists() {
            return Err(format!(
                "Audio file not found: '{}' (resolved as '{:?}')",
                asset_ref, resolved
            ));
        }

        match StaticSoundData::from_file(&resolved) {
            Ok(data) => {
                self.sound_cache.insert(asset_ref.to_string(), data.clone());
                self.sound_cache
                    .insert(resolved.to_string_lossy().to_string(), data.clone());
                Ok(data)
            }
            Err(e) => Err(format!(
                "Failed to decode audio file '{:?}': {}",
                resolved, e
            )),
        }
    }

    /// Plays a sound using specified parameters and returns a unique playback handle ID.
    pub fn play(
        &mut self,
        asset_ref: &str,
        bus: AudioBus,
        volume: f32,
        pitch: f32,
        loop_sound: bool,
        fade_in_seconds: f32,
    ) -> Result<String, String> {
        let data = self.get_or_load_sound(asset_ref)?;
        let id_val = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let playback_id = format!("audio_{}", id_val);

        // Effective volume = source volume * bus volume * master volume
        let effective_vol = self.calculate_effective_volume(bus, volume);
        let decibels = linear_to_decibels(effective_vol);

        let mut settings = StaticSoundSettings::new();
        if loop_sound {
            settings = settings.loop_region(..);
        }
        settings = settings.volume(decibels);
        settings = settings.playback_rate(PlaybackRate(pitch as f64));

        if fade_in_seconds > 0.0 {
            settings = settings.volume(Decibels::SILENCE);
            settings = settings.fade_in_tween(Some(Tween {
                start_time: StartTime::Immediate,
                duration: Duration::from_secs_f32(fade_in_seconds),
                easing: Easing::Linear,
            }));
        }

        let sound_to_play = data.with_settings(settings);

        let handle = match &mut self.backend {
            BackendManager::Cpal(mgr) => mgr
                .play(sound_to_play)
                .map_err(|e| format!("Failed to play sound: {}", e))?,
            BackendManager::Mock(mgr) => mgr
                .play(sound_to_play)
                .map_err(|e| format!("Failed to play sound on mock backend: {}", e))?,
        };

        self.active_sounds.insert(
            playback_id.clone(),
            ActiveSound {
                handle,
                bus,
                target_volume: volume,
                entity: None,
            },
        );

        Ok(playback_id)
    }

    /// Stops playback for a specific sound ID or all sounds on a bus.
    pub fn stop(
        &mut self,
        sound_id: Option<&str>,
        bus: Option<AudioBus>,
        fade_out_seconds: f32,
    ) -> bool {
        let mut stopped_any = false;
        let tween = Tween {
            start_time: StartTime::Immediate,
            duration: Duration::from_secs_f32(fade_out_seconds.max(0.0)),
            easing: Easing::Linear,
        };

        if let Some(sid) = sound_id {
            if let Some(active) = self.active_sounds.get_mut(sid) {
                if bus.is_none() || bus == Some(active.bus) {
                    active.handle.stop(tween);
                    stopped_any = true;
                }
            }
        } else if let Some(target_bus) = bus {
            for active in self.active_sounds.values_mut() {
                if active.bus == target_bus {
                    active.handle.stop(tween);
                    stopped_any = true;
                }
            }
        } else {
            // Stop all sounds
            for active in self.active_sounds.values_mut() {
                active.handle.stop(tween);
                stopped_any = true;
            }
        }

        stopped_any
    }

    /// Pauses playback for an active sound ID or bus.
    pub fn pause(
        &mut self,
        sound_id: Option<&str>,
        bus: Option<AudioBus>,
        fade_out_seconds: f32,
    ) -> bool {
        let mut paused_any = false;
        let tween = Tween {
            start_time: StartTime::Immediate,
            duration: Duration::from_secs_f32(fade_out_seconds.max(0.0)),
            easing: Easing::Linear,
        };

        if let Some(sid) = sound_id {
            if let Some(active) = self.active_sounds.get_mut(sid) {
                if bus.is_none() || bus == Some(active.bus) {
                    active.handle.pause(tween);
                    paused_any = true;
                }
            }
        } else if let Some(target_bus) = bus {
            for active in self.active_sounds.values_mut() {
                if active.bus == target_bus {
                    active.handle.pause(tween);
                    paused_any = true;
                }
            }
        } else {
            for active in self.active_sounds.values_mut() {
                active.handle.pause(tween);
                paused_any = true;
            }
        }

        paused_any
    }

    /// Resumes paused playback for an active sound ID or bus.
    pub fn resume(
        &mut self,
        sound_id: Option<&str>,
        bus: Option<AudioBus>,
        fade_in_seconds: f32,
    ) -> bool {
        let mut resumed_any = false;
        let tween = Tween {
            start_time: StartTime::Immediate,
            duration: Duration::from_secs_f32(fade_in_seconds.max(0.0)),
            easing: Easing::Linear,
        };

        if let Some(sid) = sound_id {
            if let Some(active) = self.active_sounds.get_mut(sid) {
                if bus.is_none() || bus == Some(active.bus) {
                    active.handle.resume(tween);
                    resumed_any = true;
                }
            }
        } else if let Some(target_bus) = bus {
            for active in self.active_sounds.values_mut() {
                if active.bus == target_bus {
                    active.handle.resume(tween);
                    resumed_any = true;
                }
            }
        } else {
            for active in self.active_sounds.values_mut() {
                active.handle.resume(tween);
                resumed_any = true;
            }
        }

        resumed_any
    }

    /// Crossfades current music into a new music track.
    pub fn crossfade(
        &mut self,
        new_asset_ref: &str,
        fade_duration: f32,
        loop_music: bool,
    ) -> Result<String, String> {
        // Fade out existing active music sounds
        let fade_tween = Tween {
            start_time: StartTime::Immediate,
            duration: Duration::from_secs_f32(fade_duration),
            easing: Easing::Linear,
        };

        for active in self.active_sounds.values_mut() {
            if active.bus == AudioBus::Music {
                active.handle.stop(fade_tween);
            }
        }

        // Start new music with fade in
        self.play(
            new_asset_ref,
            AudioBus::Music,
            1.0,
            1.0,
            loop_music,
            fade_duration,
        )
    }

    /// Sets the volume multiplier of an audio bus and updates active sounds on that bus.
    pub fn set_bus_volume(&mut self, bus: AudioBus, volume: f32) {
        let clamped = volume.max(0.0);
        match bus {
            AudioBus::Master => self.master_volume = clamped,
            AudioBus::Music => self.music_volume = clamped,
            AudioBus::Sfx => self.sfx_volume = clamped,
        }

        let master_vol = self.master_volume;
        let music_vol = self.music_volume;
        let sfx_vol = self.sfx_volume;

        // Recompute and update volume on all affected active sounds
        for active in self.active_sounds.values_mut() {
            if bus == AudioBus::Master || active.bus == bus {
                let bus_v = match active.bus {
                    AudioBus::Master => 1.0,
                    AudioBus::Music => music_vol,
                    AudioBus::Sfx => sfx_vol,
                };
                let eff = (active.target_volume * bus_v * master_vol).max(0.0);
                let db = linear_to_decibels(eff);
                active.handle.set_volume(
                    db,
                    Tween {
                        start_time: StartTime::Immediate,
                        duration: Duration::from_millis(20),
                        easing: Easing::Linear,
                    },
                );
            }
        }
    }

    /// Computes effective volume combining bus levels and master volume.
    pub fn calculate_effective_volume(&self, bus: AudioBus, source_volume: f32) -> f32 {
        let bus_vol = match bus {
            AudioBus::Master => 1.0,
            AudioBus::Music => self.music_volume,
            AudioBus::Sfx => self.sfx_volume,
        };
        (source_volume * bus_vol * self.master_volume).max(0.0)
    }

    /// Cleans up finished sounds from the active registry.
    pub fn cleanup_finished_sounds(&mut self) {
        self.active_sounds
            .retain(|_, active| active.handle.state() != PlaybackState::Stopped);
    }

    /// Returns count of active playing sounds.
    pub fn active_sound_count(&self) -> usize {
        self.active_sounds.len()
    }
}

/// ECS System that updates 2D spatial audio panning and distance attenuation
/// between active `AudioSource` entities and camera `AudioListener` entities.
pub fn audio_spatial_update_system(
    mut audio_server: ResMut<AudioServer>,
    sources: Query<(Entity, &AudioSource, Option<&Transform2D>)>,
    listeners: Query<(&AudioListener, Option<&Transform2D>)>,
) {
    // Find active listener position
    let listener_pos = listeners
        .iter()
        .find(|(l, _)| l.is_active)
        .and_then(|(_, tf)| tf.map(|t| t.position))
        .unwrap_or(Vec2::ZERO);

    // Update active sounds attached to spatial sources
    for (entity, source, transform_opt) in sources.iter() {
        if !source.spatial {
            continue;
        }

        let source_pos = transform_opt.map(|t| t.position).unwrap_or(Vec2::ZERO);
        let delta = source_pos - listener_pos;
        let distance = delta.length();

        // Calculate spatial pan: -1.0 (left) to 1.0 (right)
        let pan = if source.max_distance > 1e-4 {
            (delta.x / source.max_distance).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        // Distance attenuation: 1.0 / (1.0 + falloff * (distance / max_distance))
        let distance_factor = (distance / source.max_distance.max(1.0)).max(0.0);
        let attenuation = 1.0 / (1.0 + source.falloff * distance_factor);
        let spatial_vol = source.volume * attenuation;
        let effective_vol = audio_server.calculate_effective_volume(source.bus, spatial_vol);
        let db = linear_to_decibels(effective_vol);

        // Update corresponding active sounds
        for active in audio_server.active_sounds.values_mut() {
            if active.entity == Some(entity) {
                active.handle.set_panning(
                    pan,
                    Tween {
                        start_time: StartTime::Immediate,
                        duration: Duration::from_millis(30),
                        easing: Easing::Linear,
                    },
                );
                active.handle.set_volume(
                    db,
                    Tween {
                        start_time: StartTime::Immediate,
                        duration: Duration::from_millis(30),
                        easing: Easing::Linear,
                    },
                );
            }
        }
    }

    audio_server.cleanup_finished_sounds();
}

/// ECS System that auto-starts playback for newly spawned `AudioSource` components with `play_on_start = true`.
pub fn audio_source_start_system(
    _commands: Commands,
    mut audio_server: ResMut<AudioServer>,
    sources: Query<(Entity, &AudioSource)>,
) {
    for (entity, source) in sources.iter() {
        if source.play_on_start && !source.asset.is_empty() {
            // Check if already playing for this entity
            let already_playing = audio_server
                .active_sounds
                .values()
                .any(|s| s.entity == Some(entity));

            if !already_playing {
                match audio_server.play(
                    &source.asset,
                    source.bus,
                    source.volume,
                    source.pitch,
                    source.r#loop,
                    0.0,
                ) {
                    Ok(playback_id) => {
                        if let Some(active) = audio_server.active_sounds.get_mut(&playback_id) {
                            active.entity = Some(entity);
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to auto-start AudioSource on entity {:?}: {}",
                            entity,
                            e
                        );
                    }
                }
            }
        }
    }
}
