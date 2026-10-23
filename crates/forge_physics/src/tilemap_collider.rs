//! Collision generation for tilemaps by merging solid tiles into larger rectangle colliders.
//!
//! Merges contiguous horizontal and vertical solid tiles within each chunk to minimize
//! the number of collision shapes in Rapier and prevent internal edge ghost collisions.

use forge_core::components::{Collider2D, ColliderShape2D};
use forge_core::tilemap::{ChunkCoord, Tilemap};
use forge_core::Vec2;
use std::collections::{HashMap, HashSet};

/// Merged rectangle in local chunk tile coordinates `[min_x, min_y, width, height]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergedTileRect {
    /// Starting tile X within chunk (0..chunk_size).
    pub x: u32,
    /// Starting tile Y within chunk (0..chunk_size).
    pub y: u32,
    /// Width in number of tiles.
    pub width: u32,
    /// Height in number of tiles.
    pub height: u32,
}

/// Greedy rectangle merging algorithm for a 2D boolean grid of solid tiles.
///
/// Takes a `chunk_size x chunk_size` boolean mask where `true` indicates a solid tile,
/// and produces a minimal non-overlapping set of rectangles covering all solid tiles.
pub fn merge_solid_tiles(solid_grid: &[Vec<bool>], chunk_size: u32) -> Vec<MergedTileRect> {
    let cs = chunk_size as usize;
    if solid_grid.is_empty() || cs == 0 {
        return Vec::new();
    }

    let mut visited = vec![vec![false; cs]; cs];
    let mut rects = Vec::new();

    for y in 0..cs {
        for x in 0..cs {
            if solid_grid[y][x] && !visited[y][x] {
                // Determine maximum width for this row starting at (x, y)
                let mut w = 0;
                while x + w < cs && solid_grid[y][x + w] && !visited[y][x + w] {
                    w += 1;
                }

                // Expand downwards as far as full width `w` remains solid and unvisited
                let mut h = 1;
                'expand_y: while y + h < cs {
                    for test_x in 0..w {
                        if !solid_grid[y + h][x + test_x] || visited[y + h][x + test_x] {
                            break 'expand_y;
                        }
                    }
                    h += 1;
                }

                // Mark visited
                for vy in 0..h {
                    for vx in 0..w {
                        visited[y + vy][x + vx] = true;
                    }
                }

                rects.push(MergedTileRect {
                    x: x as u32,
                    y: y as u32,
                    width: w as u32,
                    height: h as u32,
                });
            }
        }
    }

    rects
}

/// Generates `Collider2D` components for a chunk's solid tiles.
///
/// Offsets are calculated in world coordinates relative to the chunk origin.
pub fn generate_chunk_colliders(
    chunk_coord: ChunkCoord,
    chunk_size: u32,
    tile_size: Vec2,
    merged_rects: &[MergedTileRect],
    collision_groups: u32,
    collision_mask: u32,
    friction: f32,
    restitution: f32,
) -> Vec<Collider2D> {
    let mut colliders = Vec::with_capacity(merged_rects.len());
    let cs = chunk_size as f32;

    // Chunk top-left in tile units
    let chunk_origin_tile_x = chunk_coord[0] as f32 * cs;
    let chunk_origin_tile_y = chunk_coord[1] as f32 * cs;

    for r in merged_rects {
        let rw = r.width as f32 * tile_size.x;
        let rh = r.height as f32 * tile_size.y;

        let half_extents = Vec2::new(rw * 0.5, rh * 0.5);

        // Center of rectangle in world units relative to tilemap origin
        let center_x = (chunk_origin_tile_x + r.x as f32) * tile_size.x + half_extents.x;
        // In 2D game coordinates (Y up or Y down), offset from origin:
        let center_y = (chunk_origin_tile_y + r.y as f32) * tile_size.y + half_extents.y;

        colliders.push(Collider2D {
            shape: ColliderShape2D::Box { half_extents },
            offset: Vec2::new(center_x, center_y),
            is_sensor: false,
            collision_groups,
            collision_mask,
            friction,
            restitution,
            density: 0.0,
        });
    }

    colliders
}

/// Tracks cached collider generation per chunk to support incremental regeneration
/// of only changed chunks.
#[derive(forge_core::ecs::Component, Debug, Default, Clone)]
pub struct TilemapColliderCache {
    /// Map of chunk coordinate -> list of generated colliders.
    pub chunk_colliders: HashMap<ChunkCoord, Vec<Collider2D>>,
    /// Set of chunk coordinates that have been modified and require collider regeneration.
    pub dirty_chunks: HashSet<ChunkCoord>,
}

impl TilemapColliderCache {
    /// Creates a new empty collider cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks a chunk as dirty, requesting regeneration on the next physics sync.
    pub fn mark_dirty(&mut self, coord: ChunkCoord) {
        self.dirty_chunks.insert(coord);
    }

    /// Regenerates colliders for all dirty chunks (or all chunks if cache is uninitialized).
    pub fn update(&mut self, tilemap: &Tilemap, is_tile_solid: impl Fn(u32) -> bool) {
        // If uninitialized, mark all existing chunks dirty
        if self.chunk_colliders.is_empty() {
            for &coord in tilemap.chunks.keys() {
                self.dirty_chunks.insert(coord);
            }
        }

        let dirty: Vec<ChunkCoord> = self.dirty_chunks.drain().collect();
        for coord in dirty {
            if let Some(chunk) = tilemap.chunks.get(&coord) {
                // Build solid mask combining all layers
                let mut solid_grid =
                    vec![vec![false; tilemap.chunk_size as usize]; tilemap.chunk_size as usize];
                for layer_name in &tilemap.layers {
                    let decoded = chunk.decode_layer(layer_name, tilemap.chunk_size);
                    for y in 0..tilemap.chunk_size as usize {
                        for x in 0..tilemap.chunk_size as usize {
                            if decoded[y][x] > 0 && is_tile_solid(decoded[y][x] as u32) {
                                solid_grid[y][x] = true;
                            }
                        }
                    }
                }

                let merged = merge_solid_tiles(&solid_grid, tilemap.chunk_size);
                let colliders = generate_chunk_colliders(
                    coord,
                    tilemap.chunk_size,
                    tilemap.tile_size,
                    &merged,
                    tilemap.collision_groups,
                    tilemap.collision_mask,
                    tilemap.friction,
                    tilemap.restitution,
                );
                self.chunk_colliders.insert(coord, colliders);
            } else {
                self.chunk_colliders.remove(&coord);
            }
        }
    }

    /// Returns an iterator over all active generated colliders across all chunks.
    pub fn all_colliders(&self) -> impl Iterator<Item = &Collider2D> {
        self.chunk_colliders.values().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_solid_tiles_simple_rect() {
        let chunk_size = 4;
        // 2x2 solid square in a 4x4 grid:
        // . . . .
        // . # # .
        // . # # .
        // . . . .
        let mut grid = vec![vec![false; 4]; 4];
        grid[1][1] = true;
        grid[1][2] = true;
        grid[2][1] = true;
        grid[2][2] = true;

        let merged = merge_solid_tiles(&grid, chunk_size);
        assert_eq!(merged.len(), 1);
        assert_eq!(
            merged[0],
            MergedTileRect {
                x: 1,
                y: 1,
                width: 2,
                height: 2,
            }
        );
    }

    #[test]
    fn test_merge_solid_tiles_disjoint_rects() {
        let chunk_size = 4;
        let mut grid = vec![vec![false; 4]; 4];
        // Horizontal bar
        grid[0][0] = true;
        grid[0][1] = true;
        grid[0][2] = true;
        // Isolated tile
        grid[3][3] = true;

        let merged = merge_solid_tiles(&grid, chunk_size);
        assert_eq!(merged.len(), 2);
    }
}
