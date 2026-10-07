//! Helper for querying ECS world entities and submitting sprite draw calls to Renderer2D.

use forge_assets::TextureStore;
use forge_core::{Camera2D, Sprite, Transform2D, World};
use forge_render::{CameraUniform, RenderContext, RenderError, Renderer2D};

/// Default background clear color: dark slate blue `[0.08, 0.09, 0.13, 1.0]`.
pub const DEFAULT_CLEAR_COLOR: [f64; 4] = [0.08, 0.09, 0.13, 1.0];

/// Traverses entities in the ECS `World`, resolves camera uniforms, queues sprite quads,
/// and executes the batched render pass.
#[allow(clippy::too_many_arguments)]
pub fn render_world_scene(
    world: &World,
    renderer: &mut Renderer2D,
    texture_store: &mut TextureStore,
    render_context: &RenderContext,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    viewport_width: f32,
    viewport_height: f32,
    clear_color: Option<[f64; 4]>,
) -> Result<(), RenderError> {
    // 1. Locate active Camera2D and its Transform2D
    let mut active_camera = Camera2D::new();
    let mut active_transform = Transform2D::IDENTITY;

    for entity_ref in world.iter_entities() {
        if let (Some(cam), Some(tf)) = (
            entity_ref.get::<Camera2D>(),
            entity_ref.get::<Transform2D>(),
        ) {
            if cam.is_active {
                active_camera = cam.clone();
                active_transform = *tf;
                break;
            }
        }
    }

    let camera_uniform = CameraUniform::from_camera(
        viewport_width,
        viewport_height,
        &active_camera,
        &active_transform,
    );

    // 2. Queue all textured sprites
    for entity_ref in world.iter_entities() {
        if let (Some(tf), Some(sprite)) =
            (entity_ref.get::<Transform2D>(), entity_ref.get::<Sprite>())
        {
            let gpu_texture = texture_store.get_or_load(&sprite.asset_id, render_context);
            renderer.draw_sprite(*tf, sprite.clone(), gpu_texture.bind_group);
        }
    }

    // 3. Submit batched draw pass
    renderer.render_to_view(target_view, target_format, clear_color, &camera_uniform)
}
