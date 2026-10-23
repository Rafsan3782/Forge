//! Importer for basic Tiled JSON files (.tmj / .json).
//!
//! Supports orthogonal maps with tile layers. Warns and ignores non-orthogonal
//! orientations, object layers, image layers, and group layers.

use forge_core::tilemap::Tilemap;
use forge_core::Vec2;
use serde::Deserialize;

/// Raw Tiled JSON map structure.
#[derive(Debug, Clone, Deserialize)]
pub struct TiledMapJson {
    /// Map orientation ("orthogonal", "isometric", etc.).
    pub orientation: String,
    /// Width of the map in tiles.
    pub width: u32,
    /// Height of the map in tiles.
    pub height: u32,
    /// Tile width in pixels.
    pub tilewidth: u32,
    /// Tile height in pixels.
    pub tileheight: u32,
    /// List of map layers.
    #[serde(default)]
    pub layers: Vec<TiledLayerJson>,
}

/// Raw Tiled layer JSON structure.
#[derive(Debug, Clone, Deserialize)]
pub struct TiledLayerJson {
    /// Layer name.
    pub name: String,
    /// Layer type ("tilelayer", "objectgroup", "imagelayer", "group").
    #[serde(rename = "type")]
    pub layer_type: String,
    /// Width of layer in tiles.
    #[serde(default)]
    pub width: u32,
    /// Height of layer in tiles.
    #[serde(default)]
    pub height: u32,
    /// 1D array of tile GIDs.
    #[serde(default)]
    pub data: Vec<i64>,
    /// Whether layer is visible.
    #[serde(default = "default_visible")]
    pub visible: bool,
    /// Opacity (0.0..1.0).
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_visible() -> bool {
    true
}

fn default_opacity() -> f32 {
    1.0
}

/// Imports a Tiled orthogonal JSON map into a Forge `Tilemap` component.
///
/// Converts 1-based Tiled GIDs into Forge tile IDs.
/// Emits warnings for unsupported features (non-orthogonal orientation or non-tile layers).
pub fn import_tiled_json(json_str: &str, tileset_id: &str) -> Result<Tilemap, String> {
    let tiled_map: TiledMapJson =
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse Tiled JSON: {}", e))?;

    if tiled_map.orientation != "orthogonal" {
        tracing::warn!(
            "Tiled map orientation '{}' is unsupported. Only 'orthogonal' maps are supported.",
            tiled_map.orientation
        );
        return Err(format!(
            "Unsupported map orientation '{}'; only 'orthogonal' is supported",
            tiled_map.orientation
        ));
    }

    let tile_size = Vec2::new(tiled_map.tilewidth as f32, tiled_map.tileheight as f32);
    let mut tilemap = Tilemap::new(tileset_id, tile_size);
    tilemap.layers.clear();

    for layer in &tiled_map.layers {
        if layer.layer_type != "tilelayer" {
            tracing::warn!(
                "Ignoring unsupported Tiled layer '{}' of type '{}'. Only 'tilelayer' is supported.",
                layer.name,
                layer.layer_type
            );
            continue;
        }

        if !tilemap.layers.contains(&layer.name) {
            tilemap.layers.push(layer.name.clone());
        }

        let lw = if layer.width > 0 {
            layer.width
        } else {
            tiled_map.width
        };
        let lh = if layer.height > 0 {
            layer.height
        } else {
            tiled_map.height
        };

        for (idx, &gid) in layer.data.iter().enumerate() {
            if gid <= 0 {
                continue;
            }

            let tx = (idx as u32 % lw) as i32;
            let ty = (idx as u32 / lw) as i32;

            if (ty as u32) >= lh {
                break;
            }

            // GID in Tiled is 1-based (0 is empty). Store gid directly as tile_id
            tilemap.set_tile(&layer.name, tx, ty, gid as i32);
        }
    }

    if tilemap.layers.is_empty() {
        tilemap.layers.push("main".to_string());
    }

    Ok(tilemap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_tiled_orthogonal() {
        let json_data = r#"{
            "orientation": "orthogonal",
            "width": 4,
            "height": 4,
            "tilewidth": 32,
            "tileheight": 32,
            "layers": [
                {
                    "name": "ground",
                    "type": "tilelayer",
                    "width": 4,
                    "height": 4,
                    "data": [
                        1, 2, 0, 0,
                        0, 0, 3, 4,
                        0, 0, 0, 0,
                        5, 0, 0, 0
                    ]
                },
                {
                    "name": "objects",
                    "type": "objectgroup",
                    "data": []
                }
            ]
        }"#;

        let tilemap = import_tiled_json(json_data, "tileset.json").expect("Failed to import");
        assert_eq!(tilemap.layers, vec!["ground"]);
        assert_eq!(tilemap.tile_size.x, 32.0);
        assert_eq!(tilemap.tile_size.y, 32.0);
        assert_eq!(tilemap.get_tile("ground", 0, 0), 1);
        assert_eq!(tilemap.get_tile("ground", 1, 0), 2);
        assert_eq!(tilemap.get_tile("ground", 2, 0), 0);
        assert_eq!(tilemap.get_tile("ground", 2, 1), 3);
        assert_eq!(tilemap.get_tile("ground", 3, 1), 4);
        assert_eq!(tilemap.get_tile("ground", 0, 3), 5);
    }

    #[test]
    fn test_import_tiled_rejects_isometric() {
        let json_data = r#"{
            "orientation": "isometric",
            "width": 4,
            "height": 4,
            "tilewidth": 32,
            "tileheight": 16,
            "layers": []
        }"#;

        let result = import_tiled_json(json_data, "tileset.json");
        assert!(result.is_err());
    }
}
