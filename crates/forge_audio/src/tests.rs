#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    use bevy_ecs::system::RunSystemOnce;
    use forge_core::components::{AudioBus, AudioListener, AudioSource, Transform2D, Vec2};
    use forge_core::World;
    use tempfile::tempdir;

    use crate::server::{
        audio_source_start_system, audio_spatial_update_system, linear_to_decibels, AudioServer,
    };

    /// Helper to write a tiny valid 16-bit PCM WAV file.
    fn write_test_wav(path: &Path, duration_samples: usize) {
        let mut file = File::create(path).expect("failed to create test wav");
        let sample_rate: u32 = 44100;
        let num_samples = duration_samples as u32;
        let data_bytes = num_samples * 2;
        let riff_size = 36 + data_bytes;

        file.write_all(b"RIFF").unwrap();
        file.write_all(&riff_size.to_le_bytes()).unwrap();
        file.write_all(b"WAVE").unwrap();
        file.write_all(b"fmt ").unwrap();
        file.write_all(&16u32.to_le_bytes()).unwrap();
        file.write_all(&1u16.to_le_bytes()).unwrap();
        file.write_all(&1u16.to_le_bytes()).unwrap();
        file.write_all(&sample_rate.to_le_bytes()).unwrap();
        file.write_all(&(sample_rate * 2).to_le_bytes()).unwrap();
        file.write_all(&2u16.to_le_bytes()).unwrap();
        file.write_all(&16u16.to_le_bytes()).unwrap();
        file.write_all(b"data").unwrap();
        file.write_all(&data_bytes.to_le_bytes()).unwrap();

        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let val = (440.0 * 2.0 * std::f32::consts::PI * t).sin() * 10000.0;
            file.write_all(&(val as i16).to_le_bytes()).unwrap();
        }
    }

    #[test]
    fn test_null_backend_creation() {
        let dir = tempdir().unwrap();
        let server = AudioServer::new_null(dir.path());
        assert!(server.is_null_backend);
        assert_eq!(server.active_sound_count(), 0);
    }

    #[test]
    fn test_bus_volume_math() {
        let dir = tempdir().unwrap();
        let mut server = AudioServer::new_null(dir.path());

        // Master = 1.0, Sfx = 1.0, Music = 1.0 initially
        assert_eq!(server.calculate_effective_volume(AudioBus::Sfx, 0.5), 0.5);
        assert_eq!(server.calculate_effective_volume(AudioBus::Music, 0.8), 0.8);

        // Adjust master bus volume
        server.set_bus_volume(AudioBus::Master, 0.5);
        assert_eq!(server.master_volume, 0.5);
        assert_eq!(server.calculate_effective_volume(AudioBus::Sfx, 1.0), 0.5);

        // Adjust sfx bus volume
        server.set_bus_volume(AudioBus::Sfx, 0.2);
        assert_eq!(server.sfx_volume, 0.2);
        // Effective = source (1.0) * sfx (0.2) * master (0.5) = 0.1
        let eff = server.calculate_effective_volume(AudioBus::Sfx, 1.0);
        assert!((eff - 0.1).abs() < 1e-5);

        // Adjust music bus volume
        server.set_bus_volume(AudioBus::Music, 0.4);
        assert_eq!(server.music_volume, 0.4);
        let eff_music = server.calculate_effective_volume(AudioBus::Music, 0.5);
        // Effective = 0.5 * 0.4 * 0.5 = 0.1
        assert!((eff_music - 0.1).abs() < 1e-5);

        // Check linear to decibels mapping
        assert_eq!(linear_to_decibels(1.0), kira::Decibels::IDENTITY);
        assert_eq!(linear_to_decibels(0.0), kira::Decibels::SILENCE);
    }

    #[test]
    fn test_play_and_stop_state_with_null_backend() {
        let dir = tempdir().unwrap();
        let wav_path = dir.path().join("ping.wav");
        write_test_wav(&wav_path, 4410); // 0.1 second wav

        let mut server = AudioServer::new_null(dir.path());

        // Play sound
        let play_result = server.play("ping.wav", AudioBus::Sfx, 1.0, 1.0, false, 0.0);
        assert!(
            play_result.is_ok(),
            "Failed to play sound: {:?}",
            play_result.err()
        );
        let playback_id = play_result.unwrap();
        assert_eq!(server.active_sound_count(), 1);

        // Play looping music sound
        let music_res = server.play("ping.wav", AudioBus::Music, 0.7, 1.0, true, 0.0);
        assert!(music_res.is_ok());
        let music_id = music_res.unwrap();
        assert_eq!(server.active_sound_count(), 2);

        // Stop specific sound
        let stopped = server.stop(Some(&playback_id), None, 0.0);
        assert!(stopped);

        // Stop by bus
        let stopped_bus = server.stop(None, Some(AudioBus::Music), 0.0);
        assert!(stopped_bus);

        // Crossfade into new track
        let crossfade_res = server.crossfade("ping.wav", 0.1, true);
        assert!(crossfade_res.is_ok());
    }

    #[test]
    fn test_missing_or_corrupt_file_handling() {
        let dir = tempdir().unwrap();
        let mut server = AudioServer::new_null(dir.path());

        // Attempt playing non-existent file
        let res = server.play("non_existent_file.wav", AudioBus::Sfx, 1.0, 1.0, false, 0.0);
        assert!(res.is_err());
        let err_msg = res.unwrap_err();
        assert!(err_msg.contains("Audio file not found"));

        // Attempt playing corrupt file
        let corrupt_path = dir.path().join("corrupt.wav");
        std::fs::write(&corrupt_path, b"not a wav file header garbage bytes").unwrap();

        let corrupt_res = server.play("corrupt.wav", AudioBus::Sfx, 1.0, 1.0, false, 0.0);
        assert!(corrupt_res.is_err());
        let corrupt_msg = corrupt_res.unwrap_err();
        assert!(corrupt_msg.contains("Failed to decode"));

        // Server remains in consistent state and still functions
        assert_eq!(server.active_sound_count(), 0);
    }

    #[test]
    fn test_audio_source_and_listener_systems() {
        let dir = tempdir().unwrap();
        let wav_path = dir.path().join("spatial.wav");
        write_test_wav(&wav_path, 4410);

        let mut world = World::new();
        let server = AudioServer::new_null(dir.path());
        world.insert_resource(server);

        // Spawn camera with AudioListener
        world.spawn((
            AudioListener { is_active: true },
            Transform2D {
                position: Vec2::new(0.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ));

        // Spawn entity with AudioSource
        let source_entity = world.spawn((
            AudioSource {
                asset: "spatial.wav".to_string(),
                volume: 1.0,
                pitch: 1.0,
                r#loop: true,
                bus: AudioBus::Sfx,
                play_on_start: true,
                spatial: true,
                max_distance: 500.0,
                falloff: 1.0,
            },
            Transform2D {
                position: Vec2::new(100.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ));

        // Run audio_source_start_system
        world.run_system_once(audio_source_start_system);

        let count = world
            .get_resource::<AudioServer>()
            .unwrap()
            .active_sound_count();
        assert_eq!(count, 1);

        // Run audio_spatial_update_system
        world.run_system_once(audio_spatial_update_system);

        let count_after = world
            .get_resource::<AudioServer>()
            .unwrap()
            .active_sound_count();
        assert_eq!(count_after, 1);
    }
}
