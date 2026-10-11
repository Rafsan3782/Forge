//! Tests for grid slicing math and atlas UV coordinates.

use forge_assets::{AtlasFrame, GridSlice, PixelRect, SpriteSheetLayout};

#[test]
fn test_grid_slicing_math() {
    let grid = GridSlice {
        frame_width: 32,
        frame_height: 32,
        margin: 0,
        spacing: 0,
        frame_count: Some(4),
    };

    // Texture 64x64 with 32x32 frames -> 2 columns, 2 rows
    // Frame 0: (0, 0, 32, 32)
    let rect0 = grid.get_frame_rect(0, 64, 64).unwrap();
    assert_eq!(rect0, PixelRect::new(0, 0, 32, 32));
    let uv0 = rect0.to_uv(64, 64);
    assert_eq!(uv0, [0.0, 0.0, 0.5, 0.5]);

    // Frame 1: (32, 0, 32, 32)
    let rect1 = grid.get_frame_rect(1, 64, 64).unwrap();
    assert_eq!(rect1, PixelRect::new(32, 0, 32, 32));
    let uv1 = rect1.to_uv(64, 64);
    assert_eq!(uv1, [0.5, 0.0, 1.0, 0.5]);

    // Frame 2: (0, 32, 32, 32)
    let rect2 = grid.get_frame_rect(2, 64, 64).unwrap();
    assert_eq!(rect2, PixelRect::new(0, 32, 32, 32));
    let uv2 = rect2.to_uv(64, 64);
    assert_eq!(uv2, [0.0, 0.5, 0.5, 1.0]);

    // Frame 3: (32, 32, 32, 32)
    let rect3 = grid.get_frame_rect(3, 64, 64).unwrap();
    assert_eq!(rect3, PixelRect::new(32, 32, 32, 32));
    let uv3 = rect3.to_uv(64, 64);
    assert_eq!(uv3, [0.5, 0.5, 1.0, 1.0]);

    // Frame 4: Out of bounds
    assert!(grid.get_frame_rect(4, 64, 64).is_none());
}

#[test]
fn test_grid_slicing_with_margin_and_spacing() {
    let grid = GridSlice {
        frame_width: 16,
        frame_height: 16,
        margin: 2,
        spacing: 4,
        frame_count: None,
    };

    // Col 0: x = 2
    // Col 1: x = 2 + 16 + 4 = 22
    let rect1 = grid.get_frame_rect(1, 64, 64).unwrap();
    assert_eq!(rect1.x, 22);
    assert_eq!(rect1.y, 2);
    assert_eq!(rect1.width, 16);
    assert_eq!(rect1.height, 16);
}

#[test]
fn test_atlas_uv_coordinates() {
    let layout = SpriteSheetLayout::Atlas {
        frames: vec![
            AtlasFrame::new("idle", PixelRect::new(0, 0, 64, 64)),
            AtlasFrame::new("run", PixelRect::new(64, 0, 64, 64)),
        ],
    };

    let uv0 = layout.get_frame_uv(0, 128, 64).unwrap();
    assert_eq!(uv0, [0.0, 0.0, 0.5, 1.0]);

    let uv1 = layout.get_frame_uv(1, 128, 64).unwrap();
    assert_eq!(uv1, [0.5, 0.0, 1.0, 1.0]);

    assert!(layout.get_frame_uv(2, 128, 64).is_none());
}
