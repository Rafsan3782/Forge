//! Tileset asset definition and per-tile physical properties.
//!
//! A tileset associates a texture asset with a uniform tile grid size and
//! custom attributes per tile index (such as `solid` or `one_way`).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Properties assigned to a specific tile index in a tileset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TileProperties {
    /// Whether this tile is solid (blocks movement and generates colliders).
    #[serde(default)]
    pub solid: bool,
    /// Whether this tile acts as a one-way platform (jump through from below).
    #[serde(default)]
    pub one_way: bool,
}

/// Tileset asset storing texture reference, tile dimensions, and per-tile flags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tileset {
    /// Asset ID or path of the texture image.
    pub texture_asset_id: String,
    /// Width and height of an individual tile in pixels `[width, height]`.
    pub tile_size: [u32; 2],
    /// Outer margin in pixels around the image.
    #[serde(default)]
    pub margin: u32,
    /// Spacing in pixels between adjacent tiles.
    #[serde(default)]
    pub spacing: u32,
    /// Optional map of tile index -> TileProperties.
    #[serde(default)]
    pub tile_properties: HashMap<u32, TileProperties>,
}

impl Tileset {
    /// Creates a new tileset with uniform tile dimensions.
    pub fn new(texture_asset_id: impl Into<String>, tile_width: u32, tile_height: u32) -> Self {
        Self {
            texture_asset_id: texture_asset_id.into(),
            tile_size: [tile_width, tile_height],
            margin: 0,
            spacing: 0,
            tile_properties: HashMap::new(),
        }
    }

    /// Checks if a tile index is marked as solid.
    pub fn is_solid(&self, tile_id: u32) -> bool {
        self.tile_properties
            .get(&tile_id)
            .map(|p| p.solid)
            .unwrap_or(false)
    }

    /// Checks if a tile index is marked as a one-way platform.
    pub fn is_one_way(&self, tile_id: u32) -> bool {
        self.tile_properties
            .get(&tile_id)
            .map(|p| p.one_way)
            .unwrap_or(false)
    }

    /// Computes normalized UV coordinates `[u_min, v_min, u_max, v_max]` for a given 0-indexed tile.
    pub fn get_tile_uv(
        &self,
        tile_index: u32,
        texture_width: u32,
        texture_height: u32,
    ) -> [f32; 4] {
        if texture_width == 0
            || texture_height == 0
            || self.tile_size[0] == 0
            || self.tile_size[1] == 0
        {
            return [0.0, 0.0, 1.0, 1.0];
        }

        let stride_x = self.tile_size[0] + self.spacing;
        let stride_y = self.tile_size[1] + self.spacing;

        let available_width = texture_width.saturating_sub(self.margin * 2);
        let columns = (available_width + self.spacing) / stride_x;
        if columns == 0 {
            return [0.0, 0.0, 1.0, 1.0];
        }

        let col = tile_index % columns;
        let row = tile_index / columns;

        let px = self.margin + col * stride_x;
        let py = self.margin + row * stride_y;

        let u_min = px as f32 / texture_width as f32;
        let v_min = py as f32 / texture_height as f32;
        let u_max = (px + self.tile_size[0]) as f32 / texture_width as f32;
        let v_max = (py + self.tile_size[1]) as f32 / texture_height as f32;

        [u_min, v_min, u_max, v_max]
    }
}
