//! Vertex layouts and quad geometry generation for 2D sprites.

use bytemuck::{Pod, Zeroable};
use forge_core::{Sprite, Transform2D};

/// Vertex format used for rendering 2D textured sprite quads.
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, PartialEq)]
pub struct SpriteVertex {
    /// 2D world position coordinates.
    pub position: [f32; 2],
    /// Texture coordinate UVs [0.0..1.0].
    pub uv: [f32; 2],
    /// Color tint multiplier [r, g, b, a].
    pub color: [f32; 4],
}

impl SpriteVertex {
    /// Creates a new sprite vertex.
    pub const fn new(position: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            position,
            uv,
            color,
        }
    }

    /// Returns the vertex buffer layout for WGPU pipeline creation.
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = [
            // position
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            // uv
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            // color
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 2]>() * 2) as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

/// Generates 4 vertices for a sprite quad transformed by `Transform2D`.
///
/// Vertices order:
/// 0: Bottom-Left
/// 1: Bottom-Right
/// 2: Top-Right
/// 3: Top-Left
pub fn generate_sprite_quad(transform: &Transform2D, sprite: &Sprite) -> [SpriteVertex; 4] {
    let hw = (sprite.size.x * 0.5) * transform.scale.x;
    let hh = (sprite.size.y * 0.5) * transform.scale.y;

    let cos = transform.rotation.cos();
    let sin = transform.rotation.sin();
    let pos = transform.position;

    let rotate_and_translate = |lx: f32, ly: f32| -> [f32; 2] {
        let rx = lx * cos - ly * sin;
        let ry = lx * sin + ly * cos;
        [rx + pos.x, ry + pos.y]
    };

    let p0 = rotate_and_translate(-hw, -hh); // bottom-left
    let p1 = rotate_and_translate(hw, -hh); // bottom-right
    let p2 = rotate_and_translate(hw, hh); // top-right
    let p3 = rotate_and_translate(-hw, hh); // top-left

    let tint = sprite.color_tint;

    let [u0, v0, u1, v1] = sprite.uv_rect.unwrap_or([0.0, 0.0, 1.0, 1.0]);

    [
        SpriteVertex::new(p0, [u0, v1], tint),
        SpriteVertex::new(p1, [u1, v1], tint),
        SpriteVertex::new(p2, [u1, v0], tint),
        SpriteVertex::new(p3, [u0, v0], tint),
    ]
}

/// Generates 4 vertices for a solid color line segment quad.
pub fn generate_line_quad(
    start: [f32; 2],
    end: [f32; 2],
    thickness: f32,
    color: [f32; 4],
) -> [SpriteVertex; 4] {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let len = (dx * dx + dy * dy).sqrt();

    let (nx, ny) = if len > 1e-6 {
        (-dy / len * thickness * 0.5, dx / len * thickness * 0.5)
    } else {
        (0.0, thickness * 0.5)
    };

    let p0 = [start[0] + nx, start[1] + ny];
    let p1 = [end[0] + nx, end[1] + ny];
    let p2 = [end[0] - nx, end[1] - ny];
    let p3 = [start[0] - nx, start[1] - ny];

    [
        SpriteVertex::new(p0, [0.0, 1.0], color),
        SpriteVertex::new(p1, [1.0, 1.0], color),
        SpriteVertex::new(p2, [1.0, 0.0], color),
        SpriteVertex::new(p3, [0.0, 0.0], color),
    ]
}

/// Generates 4 vertices for an axis-aligned tile quad.
///
/// Order matches SpriteVertex quad layout:
/// 0: Bottom-Left (min_x, min_y)
/// 1: Bottom-Right (max_x, min_y)
/// 2: Top-Right (max_x, max_y)
/// 3: Top-Left (min_x, max_y)
pub fn generate_tile_quad(
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    uv: [f32; 4],
    tint: [f32; 4],
) -> [SpriteVertex; 4] {
    let [u0, v0, u1, v1] = uv;
    [
        SpriteVertex::new([min_x, min_y], [u0, v1], tint),
        SpriteVertex::new([max_x, min_y], [u1, v1], tint),
        SpriteVertex::new([max_x, max_y], [u1, v0], tint),
        SpriteVertex::new([min_x, max_y], [u0, v0], tint),
    ]
}
