//! Helper for querying ECS world entities and submitting sprite draw calls to Renderer2D.

use forge_assets::TextureStore;
use forge_core::{Camera2D, Sprite, Transform2D, Vec2, World};
use forge_render::{CameraUniform, RenderContext, RenderError, Renderer2D};

/// Default background clear color: dark slate blue `[0.08, 0.09, 0.13, 1.0]`.
pub const DEFAULT_CLEAR_COLOR: [f64; 4] = [0.08, 0.09, 0.13, 1.0];

/// Physics debug draw toggle resource.
#[derive(forge_core::ecs::Resource, Debug, Clone, Copy, Default)]
pub struct PhysicsDebugDraw {
    /// Whether debug visualization is enabled.
    pub enabled: bool,
}

impl PhysicsDebugDraw {
    /// Toggles debug draw state.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

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
    let sheet_store = world.get_resource::<forge_assets::SpriteSheetStore>();

    let time = world.get_resource::<forge_core::Time>();
    let alpha = time.map(|t| t.alpha).unwrap_or(1.0);

    for entity_ref in world.iter_entities() {
        if let (Some(tf), Some(sprite)) =
            (entity_ref.get::<Transform2D>(), entity_ref.get::<Sprite>())
        {
            let render_tf = if let Some(prev) = entity_ref.get::<forge_core::PreviousTransform2D>()
            {
                prev.0.lerp(tf, alpha)
            } else {
                *tf
            };

            let mut resolved_sprite = sprite.clone();

            // If sprite references a frame and uv_rect is not explicitly fixed, check if there's a SpriteSheet
            if let Some(frame) = sprite.frame {
                if resolved_sprite.uv_rect.is_none() {
                    if let Some(sheets) = sheet_store {
                        if let Some(sheet) = sheets.get(&sprite.asset_id) {
                            let gpu_tex =
                                texture_store.get_or_load(&sheet.texture_asset_id, render_context);
                            if let Some(uv) =
                                sheet
                                    .layout
                                    .get_frame_uv(frame, gpu_tex.width, gpu_tex.height)
                            {
                                resolved_sprite.uv_rect = Some(uv);
                            }
                            renderer.draw_sprite(render_tf, resolved_sprite, gpu_tex.bind_group);
                            continue;
                        }
                    }
                }
            }

            let gpu_texture = texture_store.get_or_load(&sprite.asset_id, render_context);
            renderer.draw_sprite(render_tf, resolved_sprite, gpu_texture.bind_group);
        }
    }

    // 3. Optional physics debug draw of colliders and contacts
    if let Some(debug_physics) = world.get_resource::<PhysicsDebugDraw>() {
        if debug_physics.enabled {
            let default_white_tex = texture_store.white_pixel();
            for entity_ref in world.iter_entities() {
                if let (Some(tf), Some(col)) = (
                    entity_ref.get::<Transform2D>(),
                    entity_ref.get::<forge_core::Collider2D>(),
                ) {
                    let lines = forge_physics::generate_collider_debug_lines(tf, col);
                    for line in lines {
                        renderer.draw_line(
                            [line.start.x, line.start.y],
                            [line.end.x, line.end.y],
                            2.0,
                            line.color,
                            1000, // Render in front
                            default_white_tex.bind_group.clone(),
                        );
                    }
                }
            }

            // Draw contacts from PhysicsWorld2D if available
            if let Some(physics) = world.get_resource::<forge_physics::PhysicsWorld2D>() {
                let mut contact_points = Vec::new();
                for pair in physics.narrow_phase.contact_pairs() {
                    if pair.has_any_active_contact {
                        for manifold in &pair.manifolds {
                            let normal = Vec2::new(manifold.data.normal.x, manifold.data.normal.y);
                            for point in manifold.data.solver_contacts.iter() {
                                let pt = Vec2::new(point.point.x, point.point.y);
                                contact_points.push((pt, normal));
                            }
                        }
                    }
                }

                let contact_lines = forge_physics::generate_contact_debug_lines(&contact_points);
                for line in contact_lines {
                    renderer.draw_line(
                        [line.start.x, line.start.y],
                        [line.end.x, line.end.y],
                        2.0,
                        line.color,
                        1001,
                        default_white_tex.bind_group.clone(),
                    );
                }
            }
        }
    }

    // 4. Submit batched draw pass
    renderer.render_to_view(target_view, target_format, clear_color, &camera_uniform)
}
