//! Helper for querying ECS world entities and submitting sprite draw calls to Renderer2D.

use forge_assets::TextureStore;
use forge_core::{Camera2D, Sprite, Transform2D, Vec2, World};
use forge_render::{CameraUniform, RenderContext, RenderError, Renderer2D};
use std::sync::Arc;

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
    // 1. Locate Camera: if in Edit mode, use Editor camera; otherwise use active game Camera2D
    let editor_state = world.get_resource::<crate::editor::EditorState>();
    let is_edit_mode = editor_state
        .map(|e| e.play_state == crate::editor::EditorPlayState::Edit)
        .unwrap_or(false);

    let (active_camera, active_transform) = if is_edit_mode {
        if let Some(ed) = editor_state {
            (
                Camera2D {
                    zoom: ed.camera_zoom,
                    is_active: true,
                },
                Transform2D::from_position(ed.camera_position),
            )
        } else {
            (Camera2D::new(), Transform2D::IDENTITY)
        }
    } else {
        let mut cam = Camera2D::new();
        let mut tf = Transform2D::IDENTITY;
        for entity_ref in world.iter_entities() {
            if let (Some(c), Some(t)) = (
                entity_ref.get::<Camera2D>(),
                entity_ref.get::<Transform2D>(),
            ) {
                if c.is_active {
                    cam = c.clone();
                    tf = *t;
                    break;
                }
            }
        }
        (cam, tf)
    };

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

    // 2b. Queue all Tilemaps with chunk frustum culling and batching
    let tileset_store = world.get_resource::<forge_assets::TilesetStore>();
    // Compute camera orthographic visible AABB bounds in world space
    let zoom = if active_camera.zoom <= 0.0 {
        1.0
    } else {
        active_camera.zoom
    };
    let half_view_w = (viewport_width / (2.0 * zoom)).max(1.0);
    let half_view_h = (viewport_height / (2.0 * zoom)).max(1.0);
    let cam_x = active_transform.position.x;
    let cam_y = active_transform.position.y;
    let cam_min_x = cam_x - half_view_w;
    let cam_max_x = cam_x + half_view_w;
    let cam_min_y = cam_y - half_view_h;
    let cam_max_y = cam_y + half_view_h;

    for entity_ref in world.iter_entities() {
        if let (Some(tf), Some(tilemap)) = (
            entity_ref.get::<Transform2D>(),
            entity_ref.get::<forge_core::Tilemap>(),
        ) {
            let tileset = tileset_store.and_then(|ts| ts.get(&tilemap.tileset_id));
            let tex_asset_id = tileset
                .map(|ts| ts.texture_asset_id.as_str())
                .unwrap_or(tilemap.tileset_id.as_str());

            let gpu_texture = texture_store.get_or_load(tex_asset_id, render_context);
            let tex_w = gpu_texture.width;
            let tex_h = gpu_texture.height;

            let cs = tilemap.chunk_size as f32;
            let chunk_world_w = cs * tilemap.tile_size.x * tf.scale.x;
            let chunk_world_h = cs * tilemap.tile_size.y * tf.scale.y;

            for (chunk_coord, chunk) in &tilemap.chunks {
                if chunk.is_empty() {
                    continue;
                }

                // Chunk AABB in world coordinates
                let chunk_origin_x = tf.position.x + chunk_coord[0] as f32 * chunk_world_w;
                let chunk_origin_y = tf.position.y + chunk_coord[1] as f32 * chunk_world_h;
                let chunk_min_x = chunk_origin_x.min(chunk_origin_x + chunk_world_w);
                let chunk_max_x = chunk_origin_x.max(chunk_origin_x + chunk_world_w);
                let chunk_min_y = chunk_origin_y.min(chunk_origin_y + chunk_world_h);
                let chunk_max_y = chunk_origin_y.max(chunk_origin_y + chunk_world_h);

                // Frustum cull chunk AABB against camera viewport AABB
                if chunk_max_x < cam_min_x
                    || chunk_min_x > cam_max_x
                    || chunk_max_y < cam_min_y
                    || chunk_min_y > cam_max_y
                {
                    continue;
                }

                // Chunk is visible: batch all tile quads across all layers for this chunk
                let mut chunk_quads = Vec::new();
                let tint = [1.0, 1.0, 1.0, 1.0];

                for layer_name in &tilemap.layers {
                    let layer_grid = chunk.decode_layer(layer_name, tilemap.chunk_size);
                    for ly in 0..tilemap.chunk_size {
                        for lx in 0..tilemap.chunk_size {
                            let tile_id = layer_grid[ly as usize][lx as usize];
                            if tile_id <= 0 {
                                continue;
                            }

                            // 0-indexed tile id for texture uv
                            let tile_idx = (tile_id - 1) as u32;
                            let uv = if let Some(ts) = tileset {
                                ts.get_tile_uv(tile_idx, tex_w, tex_h)
                            } else {
                                [0.0, 0.0, 1.0, 1.0]
                            };

                            let tile_min_x =
                                chunk_origin_x + lx as f32 * tilemap.tile_size.x * tf.scale.x;
                            let tile_min_y =
                                chunk_origin_y + ly as f32 * tilemap.tile_size.y * tf.scale.y;
                            let tile_max_x = tile_min_x + tilemap.tile_size.x * tf.scale.x;
                            let tile_max_y = tile_min_y + tilemap.tile_size.y * tf.scale.y;

                            let quad = forge_render::generate_tile_quad(
                                tile_min_x, tile_min_y, tile_max_x, tile_max_y, uv, tint,
                            );
                            chunk_quads.push(quad);
                        }
                    }
                }

                if !chunk_quads.is_empty() {
                    renderer.draw_quad_batch(
                        chunk_quads,
                        tilemap.z_order,
                        Arc::clone(&gpu_texture.bind_group),
                    );
                }
            }
        }
    }

    // 3. Optional physics debug draw of colliders and contacts
    let default_white_tex = texture_store.white_pixel().bind_group.clone();

    if let Some(debug_physics) = world.get_resource::<PhysicsDebugDraw>() {
        if debug_physics.enabled {
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
                            default_white_tex.clone(),
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
                        default_white_tex.clone(),
                    );
                }
            }
        }
    }

    // 4. Editor Visual Overlays (Grid, Camera Frame, Colliders, Selection Outlines, Gizmos)
    if is_edit_mode {
        if let Some(ed) = editor_state {
            let z_overlay = 2000;

            // 4a. Grid
            if ed.show_grid && ed.grid_size > 0.0 {
                let grid_col = [0.2, 0.22, 0.28, 0.35];
                let axis_col = [0.35, 0.4, 0.5, 0.6];
                let gsize = ed.grid_size;

                let start_x = (cam_min_x / gsize).floor() as i32 * gsize as i32;
                let end_x = (cam_max_x / gsize).ceil() as i32 * gsize as i32;
                let start_y = (cam_min_y / gsize).floor() as i32 * gsize as i32;
                let end_y = (cam_max_y / gsize).ceil() as i32 * gsize as i32;

                // Vertical lines
                let mut x = start_x as f32;
                while x <= end_x as f32 {
                    let col = if x.abs() < 1e-4 { axis_col } else { grid_col };
                    let thick = if x.abs() < 1e-4 { 2.0 } else { 1.0 };
                    renderer.draw_line(
                        [x, cam_min_y],
                        [x, cam_max_y],
                        thick,
                        col,
                        -100,
                        default_white_tex.clone(),
                    );
                    x += gsize;
                }

                // Horizontal lines
                let mut y = start_y as f32;
                while y <= end_y as f32 {
                    let col = if y.abs() < 1e-4 { axis_col } else { grid_col };
                    let thick = if y.abs() < 1e-4 { 2.0 } else { 1.0 };
                    renderer.draw_line(
                        [cam_min_x, y],
                        [cam_max_x, y],
                        thick,
                        col,
                        -100,
                        default_white_tex.clone(),
                    );
                    y += gsize;
                }
            }

            // 4b. Game Camera Frame
            if ed.show_game_camera_frame {
                for entity_ref in world.iter_entities() {
                    if let (Some(c), Some(t)) = (
                        entity_ref.get::<Camera2D>(),
                        entity_ref.get::<Transform2D>(),
                    ) {
                        let zoom = if c.zoom <= 0.0 { 1.0 } else { c.zoom };
                        let frame_w = viewport_width / zoom;
                        let frame_h = viewport_height / zoom;
                        let hw = frame_w * 0.5;
                        let hh = frame_h * 0.5;
                        let col = [0.8, 0.8, 0.8, 0.5];

                        let p0 = [t.position.x - hw, t.position.y - hh];
                        let p1 = [t.position.x + hw, t.position.y - hh];
                        let p2 = [t.position.x + hw, t.position.y + hh];
                        let p3 = [t.position.x - hw, t.position.y + hh];

                        renderer.draw_line(p0, p1, 1.5, col, z_overlay, default_white_tex.clone());
                        renderer.draw_line(p1, p2, 1.5, col, z_overlay, default_white_tex.clone());
                        renderer.draw_line(p2, p3, 1.5, col, z_overlay, default_white_tex.clone());
                        renderer.draw_line(p3, p0, 1.5, col, z_overlay, default_white_tex.clone());
                    }
                }
            }

            // 4c. Selected Entity Outlines and Collider Outlines
            let sel_col = [1.0, 0.55, 0.15, 0.95]; // Bright Forge Orange
            for entity_id in &ed.selected_entities {
                if let Some(e) = forge_core::find_entity_by_id(world, entity_id) {
                    if let Some(tf) = world.get::<Transform2D>(e) {
                        // Draw selection box outline if sprite
                        if let Some(sprite) = world.get::<Sprite>(e) {
                            let hw = (sprite.size.x * 0.5) * tf.scale.x.abs() + 2.0;
                            let hh = (sprite.size.y * 0.5) * tf.scale.y.abs() + 2.0;

                            let cos = tf.rotation.cos();
                            let sin = tf.rotation.sin();
                            let rot = |lx: f32, ly: f32| -> [f32; 2] {
                                [
                                    tf.position.x + lx * cos - ly * sin,
                                    tf.position.y + lx * sin + ly * cos,
                                ]
                            };

                            let p0 = rot(-hw, -hh);
                            let p1 = rot(hw, -hh);
                            let p2 = rot(hw, hh);
                            let p3 = rot(-hw, hh);

                            renderer.draw_line(
                                p0,
                                p1,
                                2.0,
                                sel_col,
                                z_overlay + 10,
                                default_white_tex.clone(),
                            );
                            renderer.draw_line(
                                p1,
                                p2,
                                2.0,
                                sel_col,
                                z_overlay + 10,
                                default_white_tex.clone(),
                            );
                            renderer.draw_line(
                                p2,
                                p3,
                                2.0,
                                sel_col,
                                z_overlay + 10,
                                default_white_tex.clone(),
                            );
                            renderer.draw_line(
                                p3,
                                p0,
                                2.0,
                                sel_col,
                                z_overlay + 10,
                                default_white_tex.clone(),
                            );
                        }

                        // Draw collider outlines in Edit mode when body is selected
                        if ed.show_colliders {
                            if let Some(col) = world.get::<forge_core::Collider2D>(e) {
                                let lines = forge_physics::generate_collider_debug_lines(tf, col);
                                for line in lines {
                                    renderer.draw_line(
                                        [line.start.x, line.start.y],
                                        [line.end.x, line.end.y],
                                        2.5,
                                        [0.2, 0.85, 1.0, 0.95], // Bright cyan outline for selected collider
                                        z_overlay + 20,
                                        default_white_tex.clone(),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // 4d. Gizmos for primary selected entity
            if let Some(first_id) = ed.selected_entities.first() {
                if let Some(e) = forge_core::find_entity_by_id(world, first_id) {
                    if let Some(tf) = world.get::<Transform2D>(e) {
                        let center = tf.position;
                        let handle_size = 60.0 / ed.camera_zoom.max(0.1);
                        let x_col = [0.95, 0.25, 0.25, 1.0]; // Red X
                        let y_col = [0.25, 0.85, 0.35, 1.0]; // Green Y
                        let both_col = [1.0, 0.9, 0.2, 1.0]; // Yellow Center

                        match ed.gizmo_mode {
                            crate::editor::GizmoMode::Translate => {
                                // X arrow
                                renderer.draw_line(
                                    [center.x, center.y],
                                    [center.x + handle_size, center.y],
                                    3.0,
                                    x_col,
                                    z_overlay + 50,
                                    default_white_tex.clone(),
                                );
                                // Y arrow
                                renderer.draw_line(
                                    [center.x, center.y],
                                    [center.x, center.y + handle_size],
                                    3.0,
                                    y_col,
                                    z_overlay + 50,
                                    default_white_tex.clone(),
                                );
                                // Center square handle
                                let cs = 8.0 / ed.camera_zoom.max(0.1);
                                renderer.draw_line(
                                    [center.x, center.y],
                                    [center.x + cs, center.y],
                                    2.0,
                                    both_col,
                                    z_overlay + 51,
                                    default_white_tex.clone(),
                                );
                                renderer.draw_line(
                                    [center.x + cs, center.y],
                                    [center.x + cs, center.y + cs],
                                    2.0,
                                    both_col,
                                    z_overlay + 51,
                                    default_white_tex.clone(),
                                );
                                renderer.draw_line(
                                    [center.x + cs, center.y + cs],
                                    [center.x, center.y + cs],
                                    2.0,
                                    both_col,
                                    z_overlay + 51,
                                    default_white_tex.clone(),
                                );
                                renderer.draw_line(
                                    [center.x, center.y + cs],
                                    [center.x, center.y],
                                    2.0,
                                    both_col,
                                    z_overlay + 51,
                                    default_white_tex.clone(),
                                );
                            }
                            crate::editor::GizmoMode::Rotate => {
                                let radius = handle_size * 0.8;
                                let segments = 32;
                                let step = std::f32::consts::TAU / segments as f32;
                                let rot_col = [0.3, 0.7, 1.0, 0.95];
                                for i in 0..segments {
                                    let a1 = i as f32 * step;
                                    let a2 = (i + 1) as f32 * step;
                                    let p1 = [
                                        center.x + radius * a1.cos(),
                                        center.y + radius * a1.sin(),
                                    ];
                                    let p2 = [
                                        center.x + radius * a2.cos(),
                                        center.y + radius * a2.sin(),
                                    ];
                                    renderer.draw_line(
                                        p1,
                                        p2,
                                        2.5,
                                        rot_col,
                                        z_overlay + 50,
                                        default_white_tex.clone(),
                                    );
                                }
                            }
                            crate::editor::GizmoMode::Scale => {
                                renderer.draw_line(
                                    [center.x, center.y],
                                    [center.x + handle_size, center.y],
                                    3.0,
                                    x_col,
                                    z_overlay + 50,
                                    default_white_tex.clone(),
                                );
                                renderer.draw_line(
                                    [center.x, center.y],
                                    [center.x, center.y + handle_size],
                                    3.0,
                                    y_col,
                                    z_overlay + 50,
                                    default_white_tex.clone(),
                                );
                                let bs = 6.0 / ed.camera_zoom.max(0.1);
                                // Box at tips
                                renderer.draw_line(
                                    [center.x + handle_size - bs, center.y - bs],
                                    [center.x + handle_size + bs, center.y - bs],
                                    2.0,
                                    x_col,
                                    z_overlay + 51,
                                    default_white_tex.clone(),
                                );
                                renderer.draw_line(
                                    [center.x - bs, center.y + handle_size - bs],
                                    [center.x + bs, center.y + handle_size - bs],
                                    2.0,
                                    y_col,
                                    z_overlay + 51,
                                    default_white_tex.clone(),
                                );
                            }
                        }
                    }
                }
            }

            // 4e. Box select rectangle
            if let Some(rect) = ed.box_select_rect {
                let [x0, y0, x1, y1] = rect;
                let b_col = [0.4, 0.75, 1.0, 0.8];
                renderer.draw_line(
                    [x0, y0],
                    [x1, y0],
                    1.5,
                    b_col,
                    z_overlay + 100,
                    default_white_tex.clone(),
                );
                renderer.draw_line(
                    [x1, y0],
                    [x1, y1],
                    1.5,
                    b_col,
                    z_overlay + 100,
                    default_white_tex.clone(),
                );
                renderer.draw_line(
                    [x1, y1],
                    [x0, y1],
                    1.5,
                    b_col,
                    z_overlay + 100,
                    default_white_tex.clone(),
                );
                renderer.draw_line(
                    [x0, y1],
                    [x0, y0],
                    1.5,
                    b_col,
                    z_overlay + 100,
                    default_white_tex.clone(),
                );
            }
        }
    }

    // 5. Submit batched draw pass
    renderer.render_to_view(target_view, target_format, clear_color, &camera_uniform)
}
