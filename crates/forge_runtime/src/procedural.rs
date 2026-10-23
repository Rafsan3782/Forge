//! Procedural sprite generation for templates and test fixtures.

use image::{ImageBuffer, Rgba};
use std::path::Path;

/// Generates procedural sprites for the `templates/hello` scene:
/// - `hero.png`: A knight with gold helmet, armor, and shield.
/// - `tree.png`: A stylized tree with brown trunk and tiered green foliage.
/// - `gem.png`: A faceted glowing diamond gem.
/// - `rock.png`: A textured slate rock.
pub fn generate_hello_assets(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;

    generate_hero_sprite(&output_dir.join("hero.png"))?;
    generate_tree_sprite(&output_dir.join("tree.png"))?;
    generate_gem_sprite(&output_dir.join("gem.png"))?;
    generate_rock_sprite(&output_dir.join("rock.png"))?;

    Ok(())
}

fn generate_hero_sprite(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    const W: u32 = 32;
    const H: u32 = 32;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(W, H);

    for y in 0..H {
        for x in 0..W {
            let px = match (x, y) {
                // Gold helmet plume (top)
                (14..=17, 3..=5) => Rgba([255, 215, 0, 255]),
                // Helmet eye slit (before visor)
                (13..=18, 9) => Rgba([40, 40, 50, 255]),
                // Helmet visor
                (12..=19, 6..=11) => Rgba([192, 192, 200, 255]),
                // Torso / Blue tabard
                (10..=21, 12..=21) => {
                    if x == 10 || x == 21 || y == 12 || y == 21 {
                        Rgba([218, 165, 32, 255]) // Gold border
                    } else if (14..=17).contains(&x) {
                        Rgba([240, 240, 255, 255]) // White central cross
                    } else {
                        Rgba([30, 80, 200, 255]) // Royal blue
                    }
                }
                // Shield (left hand)
                (5..=9, 13..=21) => {
                    if x == 5 || x == 9 || y == 13 || y == 21 {
                        Rgba([218, 165, 32, 255])
                    } else {
                        Rgba([180, 30, 30, 255]) // Crimson red shield
                    }
                }
                // Sword hilt (right hand)
                (22..=26, 17) => Rgba([218, 165, 32, 255]),
                // Sword blade (right hand)
                (23..=24, 7..=22) => Rgba([220, 225, 235, 255]),
                // Legs / Boots
                (11..=14, 22..=27) | (17..=20, 22..=27) => Rgba([60, 60, 75, 255]),
                // Transparent background
                _ => Rgba([0, 0, 0, 0]),
            };
            img.put_pixel(x, y, px);
        }
    }

    img.save(path)?;
    Ok(())
}

fn generate_tree_sprite(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    const W: u32 = 32;
    const H: u32 = 48;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(W, H);

    for y in 0..H {
        for x in 0..W {
            let px = if (13..=18).contains(&x) && (32..=46).contains(&y) {
                // Brown trunk
                if x == 13 || x == 18 {
                    Rgba([90, 50, 25, 255])
                } else {
                    Rgba([139, 69, 19, 255])
                }
            } else if (8..=23).contains(&x) && (20..=33).contains(&y) {
                // Bottom canopy layer
                let dx = (x as i32 - 16).abs();
                if dx <= (y as i32 - 18) {
                    if y < 24 {
                        Rgba([60, 180, 75, 255])
                    } else {
                        Rgba([34, 139, 34, 255])
                    }
                } else {
                    Rgba([0, 0, 0, 0])
                }
            } else if (10..=21).contains(&x) && (10..=22).contains(&y) {
                // Middle canopy layer
                let dx = (x as i32 - 16).abs();
                if dx <= (y as i32 - 9) {
                    if y < 14 {
                        Rgba([80, 200, 95, 255])
                    } else {
                        Rgba([46, 150, 46, 255])
                    }
                } else {
                    Rgba([0, 0, 0, 0])
                }
            } else if (12..=19).contains(&x) && (2..=12).contains(&y) {
                // Top canopy layer
                let dx = (x as i32 - 16).abs();
                if dx <= (y as i32 - 1) {
                    Rgba([90, 215, 105, 255])
                } else {
                    Rgba([0, 0, 0, 0])
                }
            } else {
                Rgba([0, 0, 0, 0])
            };
            img.put_pixel(x, y, px);
        }
    }

    img.save(path)?;
    Ok(())
}

fn generate_gem_sprite(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    const W: u32 = 32;
    const H: u32 = 32;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(W, H);

    let cx = 16.0f32;
    let cy = 16.0f32;
    let radius = 12.0f32;

    for y in 0..H {
        for x in 0..W {
            let dx = (x as f32 + 0.5 - cx).abs();
            let dy = (y as f32 + 0.5 - cy).abs();
            let dist = dx + dy; // Diamond metric

            let px = if dist <= radius {
                if dist <= 3.0 && x < 16 && y < 16 {
                    Rgba([255, 255, 255, 255]) // Brilliant white glint
                } else if x < 16 && y < 16 {
                    Rgba([120, 240, 255, 255]) // Top-left cyan highlight
                } else if y >= 16 {
                    Rgba([0, 140, 200, 255]) // Bottom deep blue/cyan
                } else {
                    Rgba([30, 200, 240, 255]) // Mid cyan facet
                }
            } else {
                Rgba([0, 0, 0, 0])
            };
            img.put_pixel(x, y, px);
        }
    }

    img.save(path)?;
    Ok(())
}

fn generate_rock_sprite(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    const W: u32 = 32;
    const H: u32 = 24;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(W, H);

    for y in 0..H {
        for x in 0..W {
            let nx = (x as f32 - 16.0) / 14.0;
            let ny = (y as f32 - 12.0) / 9.0;
            let dist_sq = nx * nx + ny * ny;

            let px = if dist_sq <= 1.0 {
                if dist_sq < 0.4 && y < 10 {
                    Rgba([160, 165, 175, 255]) // Top highlight
                } else if y > 16 {
                    Rgba([70, 75, 85, 255]) // Shadow base
                } else if (10..=14).contains(&x) && (11..=14).contains(&y) {
                    Rgba([85, 120, 70, 255]) // Moss patch
                } else {
                    Rgba([115, 120, 130, 255]) // Main stone grey
                }
            } else {
                Rgba([0, 0, 0, 0])
            };
            img.put_pixel(x, y, px);
        }
    }

    img.save(path)?;
    Ok(())
}

/// Generates procedural tileset texture and assets for `templates/tilemap-demo`.
///
/// Creates a 64x64 tileset with 4 32x32 tiles:
/// - Tile 0 (top-left): Grass top soil (green grass line over brown dirt)
/// - Tile 1 (top-right): Stone brick / dungeon wall (grey blocks with dark mortar)
/// - Tile 2 (bottom-left): Wood platform (brown wooden planks)
/// - Tile 3 (bottom-right): Dirt filler (textured rich brown dirt)
pub fn generate_tilemap_assets(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;

    let path = output_dir.join("tileset.png");
    const W: u32 = 64;
    const H: u32 = 64;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(W, H);

    for y in 0..H {
        for x in 0..W {
            let px = match (x, y) {
                // Tile 0: Grass top (0..32, 0..32)
                (0..=31, 0..=31) => {
                    if y < 6 {
                        Rgba([76, 175, 80, 255]) // Vibrant green top
                    } else if y < 9 {
                        Rgba([56, 142, 60, 255]) // Darker green transition
                    } else if (x + y) % 7 == 0 {
                        Rgba([110, 70, 45, 255]) // Pebble speckle in dirt
                    } else {
                        Rgba([139, 90, 43, 255]) // Warm brown dirt
                    }
                }
                // Tile 1: Stone brick (32..64, 0..32)
                (32..=63, 0..=31) => {
                    let lx = x - 32;
                    let ly = y;
                    if ly == 0
                        || ly == 15
                        || ly == 31
                        || ((ly < 16 && (lx == 0 || lx == 16))
                            || (ly >= 16 && (lx == 8 || lx == 24)))
                    {
                        Rgba([45, 45, 55, 255]) // Mortar line
                    } else if (lx + ly) % 5 == 0 {
                        Rgba([180, 185, 195, 255]) // Highlight speck
                    } else {
                        Rgba([120, 125, 135, 255]) // Stone grey
                    }
                }
                // Tile 2: Wood platform (0..32, 32..64)
                (0..=31, 32..=63) => {
                    let ly = y - 32;
                    if ly == 0 || ly == 31 || ly == 10 || ly == 20 {
                        Rgba([90, 50, 25, 255]) // Dark wood groove
                    } else if ly < 4 {
                        Rgba([205, 133, 63, 255]) // Top highlight
                    } else {
                        Rgba([160, 82, 45, 255]) // Sienna plank
                    }
                }
                // Tile 3: Dirt filler (32..64, 32..64)
                (32..=63, 32..=63) => {
                    if (x * 3 + y * 7) % 11 == 0 {
                        Rgba([110, 70, 45, 255]) // Dark clump
                    } else if (x * 5 + y * 2) % 13 == 0 {
                        Rgba([165, 115, 65, 255]) // Light clump
                    } else {
                        Rgba([139, 90, 43, 255]) // Base dirt
                    }
                }
                _ => Rgba([0, 0, 0, 255]),
            };
            img.put_pixel(x, y, px);
        }
    }

    img.save(&path)?;

    // Also write hero sprite for the character controller in tilemap demo
    generate_hero_sprite(&output_dir.join("hero.png"))?;

    Ok(())
}

/// Generates procedural tones and music WAV assets for `templates/audio-demo`:
/// - `tone1.wav`: High coin ping (987 Hz / B5)
/// - `tone2.wav`: Jump blip (440 Hz -> 880 Hz sweep)
/// - `tone3.wav`: Laser zap (1200 Hz -> 200 Hz sweep)
/// - `tone4.wav`: Explosion hit (low noisy thud)
/// - `music.wav`: Looping melodic arpeggio track (C major chord sequence)
pub fn generate_audio_demo_assets(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;

    generate_coin_ping(&output_dir.join("tone1.wav"))?;
    generate_jump_blip(&output_dir.join("tone2.wav"))?;
    generate_laser_zap(&output_dir.join("tone3.wav"))?;
    generate_explosion_thud(&output_dir.join("tone4.wav"))?;
    generate_music_loop(&output_dir.join("music.wav"))?;

    Ok(())
}

/// Helper encoding 16-bit mono PCM samples into a standard RIFF WAV file.
fn write_wav_file(path: &Path, sample_rate: u32, samples: &[i16]) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(path)?;
    let num_samples = samples.len() as u32;
    let data_bytes = num_samples * 2;
    let riff_size = 36 + data_bytes;

    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&riff_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // fmt subchunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // Subchunk1Size (16 for PCM)
    file.write_all(&1u16.to_le_bytes())?; // AudioFormat (1 = PCM)
    file.write_all(&1u16.to_le_bytes())?; // NumChannels (1 = Mono)
    file.write_all(&sample_rate.to_le_bytes())?;
    let byte_rate = sample_rate * 2;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&2u16.to_le_bytes())?; // BlockAlign (NumChannels * BitsPerSample/8)
    file.write_all(&16u16.to_le_bytes())?; // BitsPerSample

    // data subchunk
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;
    for sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }

    Ok(())
}

fn generate_coin_ping(path: &Path) -> std::io::Result<()> {
    const SAMPLE_RATE: u32 = 44100;
    const DURATION_SECS: f32 = 0.35;
    let num_samples = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (-t * 12.0).exp();
        // Two-stage coin tone (B5 -> E6)
        let freq = if t < 0.08 { 987.77 } else { 1318.51 };
        let wave = (2.0 * std::f32::consts::PI * freq * t).sin();
        let val = (wave * env * 24000.0) as i16;
        samples.push(val);
    }

    write_wav_file(path, SAMPLE_RATE, &samples)
}

fn generate_jump_blip(path: &Path) -> std::io::Result<()> {
    const SAMPLE_RATE: u32 = 44100;
    const DURATION_SECS: f32 = 0.25;
    let num_samples = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / DURATION_SECS).max(0.0);
        // Pitch rises from 300 to 750 Hz
        let freq = 300.0 + 450.0 * (t / DURATION_SECS);
        // Square-ish wave for retro jump sound
        let wave = if (2.0 * std::f32::consts::PI * freq * t).sin() >= 0.0 {
            0.6
        } else {
            -0.6
        };
        let val = (wave * env * 20000.0) as i16;
        samples.push(val);
    }

    write_wav_file(path, SAMPLE_RATE, &samples)
}

fn generate_laser_zap(path: &Path) -> std::io::Result<()> {
    const SAMPLE_RATE: u32 = 44100;
    const DURATION_SECS: f32 = 0.2;
    let num_samples = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / DURATION_SECS).max(0.0).powi(2);
        // Rapid pitch descent from 1400 Hz to 200 Hz
        let freq = 200.0 + 1200.0 * (1.0 - t / DURATION_SECS).powi(2);
        let wave = (2.0 * std::f32::consts::PI * freq * t).sin();
        let val = (wave * env * 22000.0) as i16;
        samples.push(val);
    }

    write_wav_file(path, SAMPLE_RATE, &samples)
}

fn generate_explosion_thud(path: &Path) -> std::io::Result<()> {
    const SAMPLE_RATE: u32 = 44100;
    const DURATION_SECS: f32 = 0.45;
    let num_samples = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut seed = 12345u32;
    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (-t * 7.0).exp();

        // Pseudo-random noise
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((seed >> 16) as f32 / 32768.0) - 1.0;

        // Low frequency thud
        let sub = (2.0 * std::f32::consts::PI * 65.0 * t).sin();
        let mixed = 0.6 * noise + 0.5 * sub;
        let val = (mixed * env * 25000.0).clamp(-32767.0, 32767.0) as i16;
        samples.push(val);
    }

    write_wav_file(path, SAMPLE_RATE, &samples)
}

fn generate_music_loop(path: &Path) -> std::io::Result<()> {
    const SAMPLE_RATE: u32 = 44100;
    // 4 bars of 4/4 at 120 BPM = 8.0 seconds
    const DURATION_SECS: f32 = 8.0;
    let num_samples = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;
    let mut samples = vec![0i16; num_samples];

    // C - G - Am - F progression (notes in Hz)
    let notes = [
        261.63, 329.63, 392.00, 523.25, // C chord arpeggio
        196.00, 246.94, 293.66, 392.00, // G chord arpeggio
        220.00, 261.63, 329.63, 440.00, // Am chord arpeggio
        174.61, 220.00, 261.63, 349.23, // F chord arpeggio
    ];

    let note_duration = 0.5; // Eighth note at 120 BPM
    let total_notes = (DURATION_SECS / note_duration) as usize;

    for n in 0..total_notes {
        let note_freq = notes[n % notes.len()];
        let start_idx = (n as f32 * note_duration * SAMPLE_RATE as f32) as usize;
        let note_samples = (note_duration * SAMPLE_RATE as f32) as usize;

        for i in 0..note_samples {
            let idx = start_idx + i;
            if idx >= num_samples {
                break;
            }
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = (-t * 4.0).exp();
            // Pleasant synth organ tone: fundamental + second harmonic
            let wave = 0.7 * (2.0 * std::f32::consts::PI * note_freq * t).sin()
                + 0.3 * (4.0 * std::f32::consts::PI * note_freq * t).sin();
            let sample_val = (wave * env * 14000.0) as i32;

            // Mix into buffer
            let current = samples[idx] as i32;
            let sum = (current + sample_val).clamp(-32767, 32767);
            samples[idx] = sum as i16;
        }
    }

    write_wav_file(path, SAMPLE_RATE, &samples)
}
