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
