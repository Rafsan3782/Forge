//! Chunked tilemap component, tile layers, coordinate addressing, and diff-friendly JSON encoding.
//!
//! Tiles are grouped into square chunks of `chunk_size x chunk_size` tiles.
//! To ensure git diff-friendliness, chunk layers are stored as compact comma-separated row strings
//! (e.g. `"0,0,1,1"`) or run-length encoded arrays rather than huge nested arrays.

use crate::ecs::Component;
use crate::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Key representing a chunk coordinate `(chunk_x, chunk_y)`.
pub type ChunkCoord = [i32; 2];

/// A single chunk containing layer data for a `chunk_size x chunk_size` area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TilemapChunk {
    /// Chunk grid coordinate `[chunk_x, chunk_y]`.
    pub coord: ChunkCoord,
    /// Layer tile data: map of layer name -> compact row strings (one string per row of `chunk_size` tiles).
    /// Uses BTreeMap for deterministic diff-friendly JSON key ordering.
    pub layers: BTreeMap<String, Vec<String>>,
}

impl TilemapChunk {
    /// Creates a new empty chunk at the given chunk coordinates.
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            layers: BTreeMap::new(),
        }
    }

    /// Retrieves the tile ID at local chunk coordinates `(lx, ly)`.
    pub fn get_tile(&self, layer: &str, lx: u32, ly: u32, chunk_size: u32) -> i32 {
        if lx >= chunk_size || ly >= chunk_size {
            return 0;
        }

        if let Some(rows) = self.layers.get(layer) {
            if let Some(row) = rows.get(ly as usize) {
                let tiles = parse_row_string(row);
                return tiles.get(lx as usize).copied().unwrap_or(0);
            }
        }
        0
    }

    /// Sets the tile ID at local chunk coordinates `(lx, ly)`, returning the previous tile ID.
    pub fn set_tile(
        &mut self,
        layer: &str,
        lx: u32,
        ly: u32,
        tile_id: i32,
        chunk_size: u32,
    ) -> i32 {
        if lx >= chunk_size || ly >= chunk_size {
            return 0;
        }

        let rows = self
            .layers
            .entry(layer.to_string())
            .or_insert_with(|| vec!["0".to_string(); chunk_size as usize]);

        if rows.len() < chunk_size as usize {
            rows.resize(chunk_size as usize, "0".to_string());
        }

        let mut row_tiles = parse_row_string(&rows[ly as usize]);
        if row_tiles.len() < chunk_size as usize {
            row_tiles.resize(chunk_size as usize, 0);
        }

        let prev = row_tiles[lx as usize];
        row_tiles[lx as usize] = tile_id;
        rows[ly as usize] = format_row_string(&row_tiles);

        prev
    }

    /// Decodes a 2D grid `[ly][lx]` of tile IDs for a layer within this chunk.
    pub fn decode_layer(&self, layer: &str, chunk_size: u32) -> Vec<Vec<i32>> {
        let mut grid = vec![vec![0; chunk_size as usize]; chunk_size as usize];
        if let Some(rows) = self.layers.get(layer) {
            for (ly, row_str) in rows.iter().enumerate().take(chunk_size as usize) {
                let row_tiles = parse_row_string(row_str);
                for (lx, &tile) in row_tiles.iter().enumerate().take(chunk_size as usize) {
                    grid[ly][lx] = tile;
                }
            }
        }
        grid
    }

    /// Checks if this chunk contains any non-zero tiles on any layer.
    pub fn is_empty(&self) -> bool {
        for rows in self.layers.values() {
            for row in rows {
                if parse_row_string(row).iter().any(|&t| t != 0) {
                    return false;
                }
            }
        }
        true
    }
}

/// Parses a compact comma-separated string (e.g. `"0,0,1,2,0"`) into tile IDs.
pub fn parse_row_string(s: &str) -> Vec<i32> {
    if s.trim().is_empty() {
        return Vec::new();
    }
    s.split(',')
        .map(|part| part.trim().parse::<i32>().unwrap_or(0))
        .collect()
}

/// Formats a slice of tile IDs into a compact comma-separated string.
pub fn format_row_string(tiles: &[i32]) -> String {
    let parts: Vec<String> = tiles.iter().map(|t| t.to_string()).collect();
    parts.join(",")
}

/// 2D Tilemap component referencing a tileset and organizing tiles into indexed chunks.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tilemap {
    /// Asset ID of the Tileset used by this map.
    pub tileset_id: String,
    /// Size of an individual tile in world/render units `[width, height]`.
    pub tile_size: Vec2,
    /// Number of tiles along one dimension of a chunk (e.g. 16 means 16x16 tiles per chunk).
    #[serde(default = "default_chunk_size")]
    pub chunk_size: u32,
    /// List of ordered layer names (from bottom-most to top-most).
    #[serde(default = "default_layers")]
    pub layers: Vec<String>,
    /// Map of chunk coordinate `[chunk_x, chunk_y]` -> Chunk data.
    /// Uses BTreeMap for deterministic diff-friendly JSON key ordering.
    #[serde(
        default,
        serialize_with = "serialize_chunks",
        deserialize_with = "deserialize_chunks"
    )]
    pub chunks: BTreeMap<ChunkCoord, TilemapChunk>,
    /// Collision layer bitmask for generated colliders.
    #[serde(default = "default_collision_groups")]
    pub collision_groups: u32,
    /// Collision interaction filter mask.
    #[serde(default = "default_collision_groups")]
    pub collision_mask: u32,
    /// Friction of generated colliders.
    #[serde(default = "default_friction")]
    pub friction: f32,
    /// Restitution of generated colliders.
    #[serde(default)]
    pub restitution: f32,
    /// Rendering z-order offset.
    #[serde(default)]
    pub z_order: i32,
}

fn default_chunk_size() -> u32 {
    16
}

fn default_layers() -> Vec<String> {
    vec!["main".to_string()]
}

fn default_collision_groups() -> u32 {
    0xFFFF_FFFF
}

fn default_friction() -> f32 {
    0.5
}

fn serialize_chunks<S>(
    chunks: &BTreeMap<ChunkCoord, TilemapChunk>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(chunks.len()))?;
    for (k, v) in chunks {
        let key_str = format!("[{}, {}]", k[0], k[1]);
        map.serialize_entry(&key_str, v)?;
    }
    map.end()
}

fn deserialize_chunks<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<ChunkCoord, TilemapChunk>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let raw_map: std::collections::HashMap<String, TilemapChunk> =
        serde::Deserialize::deserialize(deserializer)?;
    let mut chunks = BTreeMap::new();

    for (k_str, chunk) in raw_map {
        let trimmed = k_str
            .trim()
            .trim_matches(|c| c == '[' || c == ']' || c == '(' || c == ')');
        let parts: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                chunks.insert([x, y], chunk);
                continue;
            }
        }
        return Err(D::Error::custom(format!(
            "Invalid chunk coordinate key: '{}', expected format '[x, y]'",
            k_str
        )));
    }

    Ok(chunks)
}

impl Default for Tilemap {
    fn default() -> Self {
        Self {
            tileset_id: String::new(),
            tile_size: Vec2::new(16.0, 16.0),
            chunk_size: 16,
            layers: default_layers(),
            chunks: BTreeMap::new(),
            collision_groups: 0xFFFF_FFFF,
            collision_mask: 0xFFFF_FFFF,
            friction: 0.5,
            restitution: 0.0,
            z_order: 0,
        }
    }
}

impl Tilemap {
    /// Creates a new empty tilemap.
    pub fn new(tileset_id: impl Into<String>, tile_size: Vec2) -> Self {
        Self {
            tileset_id: tileset_id.into(),
            tile_size,
            ..Default::default()
        }
    }

    /// Converts world tile coordinate `(tx, ty)` into `(chunk_coord, local_coord)`.
    pub fn tile_to_chunk_coord(&self, tx: i32, ty: i32) -> (ChunkCoord, [u32; 2]) {
        let cs = self.chunk_size as i32;

        let cx = tx.div_euclid(cs);
        let cy = ty.div_euclid(cs);

        let lx = tx.rem_euclid(cs) as u32;
        let ly = ty.rem_euclid(cs) as u32;

        ([cx, cy], [lx, ly])
    }

    /// Converts `(chunk_coord, local_coord)` into world tile coordinate `(tx, ty)`.
    pub fn chunk_to_tile_coord(&self, chunk_coord: ChunkCoord, lx: u32, ly: u32) -> (i32, i32) {
        let cs = self.chunk_size as i32;
        let tx = chunk_coord[0] * cs + lx as i32;
        let ty = chunk_coord[1] * cs + ly as i32;
        (tx, ty)
    }

    /// Gets the tile ID at global tile coordinate `(tx, ty)` on the given layer.
    pub fn get_tile(&self, layer: &str, tx: i32, ty: i32) -> i32 {
        let (chunk_coord, [lx, ly]) = self.tile_to_chunk_coord(tx, ty);
        if let Some(chunk) = self.chunks.get(&chunk_coord) {
            chunk.get_tile(layer, lx, ly, self.chunk_size)
        } else {
            0
        }
    }

    /// Sets the tile ID at global tile coordinate `(tx, ty)`, returning the previous tile ID.
    pub fn set_tile(&mut self, layer: &str, tx: i32, ty: i32, tile_id: i32) -> i32 {
        let (chunk_coord, [lx, ly]) = self.tile_to_chunk_coord(tx, ty);
        let chunk_size = self.chunk_size;
        let chunk = self
            .chunks
            .entry(chunk_coord)
            .or_insert_with(|| TilemapChunk::new(chunk_coord));
        chunk.set_tile(layer, lx, ly, tile_id, chunk_size)
    }

    /// Fills a rectangular region of tiles `[min_x..=max_x, min_y..=max_y]` on the specified layer.
    /// Returns the number of modified tiles and affected chunk count.
    pub fn fill_rect(
        &mut self,
        layer: &str,
        min: [i32; 2],
        max: [i32; 2],
        tile_id: i32,
    ) -> (u32, u32) {
        let (min_x, max_x) = if min[0] <= max[0] {
            (min[0], max[0])
        } else {
            (max[0], min[0])
        };
        let (min_y, max_y) = if min[1] <= max[1] {
            (min[1], max[1])
        } else {
            (max[1], min[1])
        };

        let mut tiles_modified = 0;
        let mut affected_chunks = std::collections::HashSet::new();

        for ty in min_y..=max_y {
            for tx in min_x..=max_x {
                let (chunk_coord, _) = self.tile_to_chunk_coord(tx, ty);
                affected_chunks.insert(chunk_coord);
                self.set_tile(layer, tx, ty, tile_id);
                tiles_modified += 1;
            }
        }

        (tiles_modified, affected_chunks.len() as u32)
    }

    /// Clears tiles across the entire map, or from a specific layer.
    /// Returns the number of chunks cleared.
    pub fn clear_map(&mut self, layer: Option<&str>) -> u32 {
        let count = self.chunks.len() as u32;
        if let Some(target_layer) = layer {
            for chunk in self.chunks.values_mut() {
                chunk.layers.remove(target_layer);
            }
            self.chunks.retain(|_, chunk| !chunk.is_empty());
        } else {
            self.chunks.clear();
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_addressing() {
        let tm = Tilemap::new("test", Vec2::new(16.0, 16.0));
        assert_eq!(tm.chunk_size, 16);

        // Positive coordinates
        let (c, l) = tm.tile_to_chunk_coord(0, 0);
        assert_eq!(c, [0, 0]);
        assert_eq!(l, [0, 0]);

        let (c, l) = tm.tile_to_chunk_coord(15, 15);
        assert_eq!(c, [0, 0]);
        assert_eq!(l, [15, 15]);

        let (c, l) = tm.tile_to_chunk_coord(16, 32);
        assert_eq!(c, [1, 2]);
        assert_eq!(l, [0, 0]);

        // Negative coordinates
        let (c, l) = tm.tile_to_chunk_coord(-1, -1);
        assert_eq!(c, [-1, -1]);
        assert_eq!(l, [15, 15]);

        let (c, l) = tm.tile_to_chunk_coord(-16, -16);
        assert_eq!(c, [-1, -1]);
        assert_eq!(l, [0, 0]);

        let (c, l) = tm.tile_to_chunk_coord(-17, -1);
        assert_eq!(c, [-2, -1]);
        assert_eq!(l, [15, 15]);

        // Roundtrip chunk_to_tile_coord
        let (tx, ty) = tm.chunk_to_tile_coord([-2, 3], 5, 7);
        assert_eq!(tx, -2 * 16 + 5);
        assert_eq!(ty, 3 * 16 + 7);
    }

    #[test]
    fn test_set_tile_modifies_single_chunk() {
        let mut tm = Tilemap::new("test", Vec2::new(16.0, 16.0));
        assert_eq!(tm.chunks.len(), 0);

        let prev = tm.set_tile("main", 5, 5, 42);
        assert_eq!(prev, 0);
        assert_eq!(tm.chunks.len(), 1);
        assert!(tm.chunks.contains_key(&[0, 0]));

        assert_eq!(tm.get_tile("main", 5, 5), 42);
        assert_eq!(tm.get_tile("main", 6, 5), 0);

        // Modify in same chunk
        let prev = tm.set_tile("main", 5, 5, 99);
        assert_eq!(prev, 42);
        assert_eq!(tm.chunks.len(), 1);

        // Modify in neighboring chunk
        tm.set_tile("main", 17, 5, 1);
        assert_eq!(tm.chunks.len(), 2);
        assert!(tm.chunks.contains_key(&[1, 0]));
    }
}
