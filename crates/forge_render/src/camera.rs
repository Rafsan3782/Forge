//! 2D Orthographic camera projection and uniform definitions.

use bytemuck::{Pod, Zeroable};
use forge_core::{Camera2D, Transform2D, Vec2};

/// Uniform buffer structure uploaded to the GPU representing camera view-projection.
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    /// 4x4 column-major view-projection matrix.
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl CameraUniform {
    /// Calculates the 2D orthographic view-projection matrix from viewport size and camera state.
    ///
    /// The coordinate system is Cartesian: origin is at camera position, +X extends right,
    /// and +Y extends up. The visible width and height in world space are scaled inversely by zoom.
    pub fn from_camera(
        viewport_width: f32,
        viewport_height: f32,
        camera: &Camera2D,
        transform: &Transform2D,
    ) -> Self {
        let width = if viewport_width.abs() < 1e-4 {
            1.0
        } else {
            viewport_width
        };
        let height = if viewport_height.abs() < 1e-4 {
            1.0
        } else {
            viewport_height
        };

        let zoom = if camera.zoom <= 0.0 { 1.0 } else { camera.zoom };

        let sx = 2.0 * zoom / width;
        let sy = 2.0 * zoom / height;

        let cos = transform.rotation.cos();
        let sin = transform.rotation.sin();

        let pos = transform.position;
        let tx = -(pos.x * cos + pos.y * sin) * sx;
        let ty = -(-pos.x * sin + pos.y * cos) * sy;

        // Column-major 4x4 matrix for WGSL
        Self {
            view_proj: [
                [sx * cos, -sy * sin, 0.0, 0.0],
                [sx * sin, sy * cos, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [tx, ty, 0.0, 1.0],
            ],
        }
    }

    /// Creates an orthographic camera matrix centered at origin for the given viewport size.
    pub fn from_viewport(width: f32, height: f32) -> Self {
        Self::from_camera(
            width,
            height,
            &Camera2D::new(),
            &Transform2D::from_position(Vec2::ZERO),
        )
    }
}
