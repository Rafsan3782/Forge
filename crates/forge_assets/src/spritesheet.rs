//! Sprite sheet slicing (uniform grid and JSON atlas layouts).
//!
//! Provides UV frame calculation for:
//! 1. Uniform grid slicing (`frame_width`, `frame_height`, `margin`, `spacing`).
//! 2. Arbitrary JSON frame atlas layouts (named or indexed sub-rectangles).

use crate::error::AssetError;
use serde::{Deserialize, Serialize};

/// Rectangle in pixel coordinates `[x, y, width, height]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PixelRect {
    /// Horizontal pixel offset from the left.
    pub x: u32,
    /// Vertical pixel offset from the top.
    pub y: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl PixelRect {
    /// Creates a new pixel rectangle.
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Computes normalized UV coordinates `[u_min, v_min, u_max, v_max]` given the total texture dimensions.
    pub fn to_uv(self, texture_width: u32, texture_height: u32) -> [f32; 4] {
        if texture_width == 0 || texture_height == 0 {
            return [0.0, 0.0, 1.0, 1.0];
        }

        let tw = texture_width as f32;
        let th = texture_height as f32;

        let u_min = self.x as f32 / tw;
        let v_min = self.y as f32 / th;
        let u_max = (self.x + self.width) as f32 / tw;
        let v_max = (self.y + self.height) as f32 / th;

        [u_min, v_min, u_max, v_max]
    }
}

/// Grid slicing specification for evenly spaced sprite sheets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GridSlice {
    /// Width of an individual frame in pixels.
    pub frame_width: u32,
    /// Height of an individual frame in pixels.
    pub frame_height: u32,
    /// Outer margin in pixels around the sheet.
    #[serde(default)]
    pub margin: u32,
    /// Spacing / padding between neighboring frames in pixels.
    #[serde(default)]
    pub spacing: u32,
    /// Total number of frames, or None to calculate automatically from sheet dimensions.
    #[serde(default)]
    pub frame_count: Option<u32>,
}

impl GridSlice {
    /// Computes the pixel rectangle for a frame index given total texture dimensions.
    pub fn get_frame_rect(
        &self,
        frame_index: u32,
        texture_width: u32,
        texture_height: u32,
    ) -> Option<PixelRect> {
        let cell_w = self.frame_width + self.spacing;
        let cell_h = self.frame_height + self.spacing;

        if cell_w == 0 || cell_h == 0 {
            return None;
        }

        let usable_w = texture_width.saturating_sub(self.margin * 2);
        let columns = (usable_w + self.spacing) / cell_w;
        if columns == 0 {
            return None;
        }

        let usable_h = texture_height.saturating_sub(self.margin * 2);
        let rows = (usable_h + self.spacing) / cell_h;
        let max_frames = columns * rows;

        if let Some(count) = self.frame_count {
            if frame_index >= count {
                return None;
            }
        } else if frame_index >= max_frames {
            return None;
        }

        let col = frame_index % columns;
        let row = frame_index / columns;

        let x = self.margin + col * cell_w;
        let y = self.margin + row * cell_h;

        Some(PixelRect::new(x, y, self.frame_width, self.frame_height))
    }
}

/// A frame definition in a JSON sprite atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtlasFrame {
    /// Name or tag of the frame (optional).
    #[serde(default)]
    pub name: Option<String>,
    /// Pixel bounds of the frame.
    pub rect: PixelRect,
}

impl AtlasFrame {
    /// Creates a named atlas frame.
    pub fn new(name: impl Into<String>, rect: PixelRect) -> Self {
        Self {
            name: Some(name.into()),
            rect,
        }
    }

    /// Creates an unnamed atlas frame.
    pub fn unnamed(rect: PixelRect) -> Self {
        Self { name: None, rect }
    }
}

/// Sprite sheet layout specification supporting both grid slicing and arbitrary atlas definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpriteSheetLayout {
    /// Uniform grid slicing.
    Grid(GridSlice),
    /// Frame atlas list/map.
    Atlas {
        /// Ordered list of frames.
        frames: Vec<AtlasFrame>,
    },
}

impl SpriteSheetLayout {
    /// Computes the normalized UV bounds `[u_min, v_min, u_max, v_max]` for a given frame index.
    pub fn get_frame_uv(
        &self,
        frame_index: u32,
        texture_width: u32,
        texture_height: u32,
    ) -> Option<[f32; 4]> {
        match self {
            SpriteSheetLayout::Grid(grid) => {
                let rect = grid.get_frame_rect(frame_index, texture_width, texture_height)?;
                Some(rect.to_uv(texture_width, texture_height))
            }
            SpriteSheetLayout::Atlas { frames } => {
                let frame = frames.get(frame_index as usize)?;
                Some(frame.rect.to_uv(texture_width, texture_height))
            }
        }
    }

    /// Returns the total frame count if known.
    pub fn frame_count(&self, texture_width: u32, texture_height: u32) -> usize {
        match self {
            SpriteSheetLayout::Grid(grid) => {
                if let Some(count) = grid.frame_count {
                    count as usize
                } else {
                    let cell_w = grid.frame_width + grid.spacing;
                    let cell_h = grid.frame_height + grid.spacing;
                    if cell_w == 0 || cell_h == 0 {
                        return 0;
                    }
                    let cols =
                        (texture_width.saturating_sub(grid.margin * 2) + grid.spacing) / cell_w;
                    let rows =
                        (texture_height.saturating_sub(grid.margin * 2) + grid.spacing) / cell_h;
                    (cols * rows) as usize
                }
            }
            SpriteSheetLayout::Atlas { frames } => frames.len(),
        }
    }
}

/// Standalone SpriteSheet asset defining texture asset ID and layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheet {
    /// Asset ID of the underlying texture image.
    pub texture_asset_id: String,
    /// Slicing layout.
    pub layout: SpriteSheetLayout,
}

impl SpriteSheet {
    /// Creates a grid-sliced sprite sheet.
    pub fn new_grid(texture_asset_id: impl Into<String>, grid: GridSlice) -> Self {
        Self {
            texture_asset_id: texture_asset_id.into(),
            layout: SpriteSheetLayout::Grid(grid),
        }
    }

    /// Loads a SpriteSheet definition from a JSON string.
    pub fn from_json_str(json: &str) -> Result<Self, AssetError> {
        let sheet = serde_json::from_str(json)?;
        Ok(sheet)
    }

    /// Saves a SpriteSheet definition to a JSON string.
    pub fn to_json_string(&self) -> Result<String, AssetError> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }
}
