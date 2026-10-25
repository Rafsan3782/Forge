//! Editor interaction systems: Editor camera, picking, selection, gizmos, and play/edit mode.

use forge_core::components::{Collider2D, Sprite, Transform2D};
use forge_core::{Camera2D, CommandAuthor, CommandStack, Entity, Scene, SceneCommand, Vec2, World};
use forge_input::{InputState, KeyCode, MouseButton};

/// Editor state mode: Edit (paused simulation, tools active), Playing (full simulation), Paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorPlayState {
    /// Edit mode: physics and scripts do not advance. Tools and gizmos active.
    #[default]
    Edit,
    /// Play mode: full simulation active.
    Playing,
    /// Paused during Play mode.
    Paused,
}

/// Active transform gizmo mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    /// Translation gizmo (W key)
    #[default]
    Translate,
    /// Rotation gizmo (E key)
    Rotate,
    /// Scale gizmo (R key)
    Scale,
}

/// Active gizmo drag constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoDragAxis {
    X,
    Y,
    Both,
    Rotation,
}

/// Editor state resource stored in ECS World.
#[derive(Debug, forge_core::ecs::Resource)]
pub struct EditorState {
    /// Play mode state
    pub play_state: EditorPlayState,
    /// Scene snapshot taken before entering Play mode (exact restore on Stop)
    pub play_snapshot: Option<Scene>,
    /// Number of frames requested to step while in Paused state
    pub step_frames_remaining: u32,

    // Editor Camera (independent from game Camera2D)
    /// Editor camera translation position
    pub camera_position: Vec2,
    /// Editor camera zoom
    pub camera_zoom: f32,
    /// Whether editor camera panning is active
    pub is_panning: bool,
    /// Last mouse screen position during pan
    pub last_pan_mouse_pos: Vec2,

    // Visual options
    /// Show grid in Edit mode
    pub show_grid: bool,
    /// Grid spacing in world units
    pub grid_size: f32,
    /// Show game camera frame
    pub show_game_camera_frame: bool,
    /// Show collider outlines on selected bodies
    pub show_colliders: bool,

    // Selection
    /// Currently selected entity IDs
    pub selected_entities: Vec<String>,
    /// Box selection start point in world coordinates
    pub box_select_start: Option<Vec2>,
    /// Current box select rect [min_x, min_y, max_x, max_y] in world units
    pub box_select_rect: Option<[f32; 4]>,

    // Gizmos
    /// Current gizmo tool mode
    pub gizmo_mode: GizmoMode,
    /// Active drag state if currently manipulating gizmo
    pub active_gizmo_drag: Option<GizmoDragState>,
    /// Snap to grid toggle
    pub snap_to_grid: bool,
    /// Angle snap toggle (e.g. 15 degrees)
    pub snap_to_angle: bool,
}

/// Active gizmo manipulation state.
#[derive(Debug, Clone)]
pub struct GizmoDragState {
    pub axis: GizmoDragAxis,
    pub start_world_pos: Vec2,
    pub start_mouse_world: Vec2,
    pub original_transforms: Vec<(String, Transform2D)>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            play_state: EditorPlayState::Edit,
            play_snapshot: None,
            step_frames_remaining: 0,
            camera_position: Vec2::ZERO,
            camera_zoom: 1.0,
            is_panning: false,
            last_pan_mouse_pos: Vec2::ZERO,
            show_grid: true,
            grid_size: 32.0,
            show_game_camera_frame: true,
            show_colliders: true,
            selected_entities: Vec::new(),
            box_select_start: None,
            box_select_rect: None,
            gizmo_mode: GizmoMode::Translate,
            active_gizmo_drag: None,
            snap_to_grid: false,
            snap_to_angle: false,
        }
    }
}

impl EditorState {
    /// Creates a new editor state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts screen pixel coordinates to world coordinates using the Editor Camera.
    pub fn screen_to_world(&self, screen_pos: Vec2, viewport_w: f32, viewport_h: f32) -> Vec2 {
        let vw = if viewport_w.abs() < 1e-4 {
            800.0
        } else {
            viewport_w
        };
        let vh = if viewport_h.abs() < 1e-4 {
            600.0
        } else {
            viewport_h
        };

        // Normalize screen coordinates from [0..vw, 0..vh] to [-1..1, -1..1] (Cartesian Y up)
        let ndc_x = (screen_pos.x - vw * 0.5) / (vw * 0.5);
        let ndc_y = (vh * 0.5 - screen_pos.y) / (vh * 0.5);

        let zoom = if self.camera_zoom <= 0.0 {
            1.0
        } else {
            self.camera_zoom
        };
        let half_w = (vw * 0.5) / zoom;
        let half_h = (vh * 0.5) / zoom;

        let vx = ndc_x * half_w;
        let vy = ndc_y * half_h;

        Vec2::new(self.camera_position.x + vx, self.camera_position.y + vy)
    }

    /// Converts world coordinates to screen pixel coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2, viewport_w: f32, viewport_h: f32) -> Vec2 {
        let vw = if viewport_w.abs() < 1e-4 {
            800.0
        } else {
            viewport_w
        };
        let vh = if viewport_h.abs() < 1e-4 {
            600.0
        } else {
            viewport_h
        };

        let zoom = if self.camera_zoom <= 0.0 {
            1.0
        } else {
            self.camera_zoom
        };
        let vx = world_pos.x - self.camera_position.x;
        let vy = world_pos.y - self.camera_position.y;

        let half_w = (vw * 0.5) / zoom;
        let half_h = (vh * 0.5) / zoom;

        let ndc_x = vx / half_w;
        let ndc_y = vy / half_h;

        let sx = ndc_x * (vw * 0.5) + (vw * 0.5);
        let sy = (vh * 0.5) - ndc_y * (vh * 0.5);

        Vec2::new(sx, sy)
    }

    /// Snaps a position to grid if grid snapping is active.
    pub fn snap_position(&self, pos: Vec2) -> Vec2 {
        if self.snap_to_grid && self.grid_size > 0.0 {
            let x = (pos.x / self.grid_size).round() * self.grid_size;
            let y = (pos.y / self.grid_size).round() * self.grid_size;
            Vec2::new(x, y)
        } else {
            pos
        }
    }

    /// Snaps an angle in radians to step increments (15 deg) if angle snapping is active.
    pub fn snap_rotation(&self, angle: f32) -> f32 {
        if self.snap_to_angle {
            let step = 15.0_f32.to_radians();
            (angle / step).round() * step
        } else {
            angle
        }
    }

    /// Starts play mode by snapshotting the scene.
    pub fn start_play(&mut self, world: &World) -> Result<(), String> {
        if self.play_state == EditorPlayState::Edit {
            let snapshot = Scene::from_world(world, "play_snapshot").map_err(|e| e.to_string())?;
            self.play_snapshot = Some(snapshot);
            self.play_state = EditorPlayState::Playing;
            self.step_frames_remaining = 0;
            self.active_gizmo_drag = None;
            self.box_select_start = None;
            self.box_select_rect = None;
        }
        Ok(())
    }

    /// Pauses play mode.
    pub fn pause_play(&mut self) {
        if self.play_state == EditorPlayState::Playing {
            self.play_state = EditorPlayState::Paused;
        } else if self.play_state == EditorPlayState::Paused {
            self.play_state = EditorPlayState::Playing;
        }
    }

    /// Steps simulation by N frames.
    pub fn step_play(&mut self, frames: u32) {
        if self.play_state == EditorPlayState::Paused {
            self.step_frames_remaining += frames.max(1);
        }
    }

    /// Stops play mode and restores snapshot exactly.
    pub fn stop_play(&mut self, world: &mut World) -> Result<(), String> {
        if let Some(snapshot) = self.play_snapshot.take() {
            // Clear current entities
            world.clear_entities();
            // Restore snapshot
            snapshot
                .spawn_into_world(world)
                .map_err(|e| e.to_string())?;
        }
        self.play_state = EditorPlayState::Edit;
        self.step_frames_remaining = 0;
        self.active_gizmo_drag = None;
        self.box_select_start = None;
        self.box_select_rect = None;
        Ok(())
    }
}

/// Pick result for clicking on entities in the scene.
#[derive(Debug, Clone)]
pub struct PickResult {
    /// Entity ID hit
    pub entity_id: String,
    /// ECS entity handle
    pub entity: Entity,
    /// Z-order of the hit object
    pub z_order: i32,
    /// Distance or priority
    pub hit_area: f32,
}

/// Queries all entities under the cursor in world coordinates, returning them sorted topmost first.
pub fn pick_entities_at(world: &World, world_pos: Vec2) -> Vec<PickResult> {
    let mut hits = Vec::new();

    for entity_ref in world.iter_entities() {
        let entity_id = match entity_ref.get::<forge_core::EntityId>() {
            Some(id) => id.0.clone(),
            None => continue,
        };

        let transform = match entity_ref.get::<Transform2D>() {
            Some(tf) => *tf,
            None => continue,
        };

        // 1. Check Sprite Quad hit test (accounting for rotation and scale)
        if let Some(sprite) = entity_ref.get::<Sprite>() {
            let hw = (sprite.size.x * 0.5) * transform.scale.x.abs();
            let hh = (sprite.size.y * 0.5) * transform.scale.y.abs();

            // Transform world_pos into local sprite space
            let dx = world_pos.x - transform.position.x;
            let dy = world_pos.y - transform.position.y;
            let cos = (-transform.rotation).cos();
            let sin = (-transform.rotation).sin();
            let lx = dx * cos - dy * sin;
            let ly = dx * sin + dy * cos;

            if lx.abs() <= hw && ly.abs() <= hh {
                hits.push(PickResult {
                    entity_id: entity_id.clone(),
                    entity: entity_ref.id(),
                    z_order: sprite.z_order,
                    hit_area: (hw * 2.0) * (hh * 2.0),
                });
                continue;
            }
        }

        // 2. Check Tilemap hit test
        if let Some(tilemap) = entity_ref.get::<forge_core::Tilemap>() {
            let rel_x = world_pos.x - transform.position.x;
            let rel_y = world_pos.y - transform.position.y;
            let tile_w = tilemap.tile_size.x * transform.scale.x;
            let tile_h = tilemap.tile_size.y * transform.scale.y;

            if tile_w.abs() > 1e-4 && tile_h.abs() > 1e-4 {
                let tx = (rel_x / tile_w).floor() as i32;
                let ty = (rel_y / tile_h).floor() as i32;

                // Check all layers for a non-empty tile at (tx, ty)
                let mut found_tile = false;
                for layer in &tilemap.layers {
                    if tilemap.get_tile(layer, tx, ty) > 0 {
                        found_tile = true;
                        break;
                    }
                }

                if found_tile {
                    hits.push(PickResult {
                        entity_id: entity_id.clone(),
                        entity: entity_ref.id(),
                        z_order: tilemap.z_order,
                        hit_area: 1.0,
                    });
                }
            }
        }
    }

    // Sort descending by z_order; for equal z_order, prefer smaller hit_area (topmost/foreground item)
    hits.sort_by(|a, b| {
        b.z_order.cmp(&a.z_order).then_with(|| {
            a.hit_area
                .partial_cmp(&b.hit_area)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    hits
}

/// Finds all entities intersecting the given world-space bounding box `[min_x, min_y, max_x, max_y]`.
pub fn pick_entities_in_rect(world: &World, rect: [f32; 4]) -> Vec<String> {
    let [min_x, min_y, max_x, max_y] = [
        rect[0].min(rect[2]),
        rect[1].min(rect[3]),
        rect[0].max(rect[2]),
        rect[1].max(rect[3]),
    ];

    let mut selected = Vec::new();

    for entity_ref in world.iter_entities() {
        let entity_id = match entity_ref.get::<forge_core::EntityId>() {
            Some(id) => id.0.clone(),
            None => continue,
        };

        if let Some(tf) = entity_ref.get::<Transform2D>() {
            // If sprite, check AABB overlap
            if let Some(sprite) = entity_ref.get::<Sprite>() {
                let hw = (sprite.size.x * 0.5) * tf.scale.x.abs();
                let hh = (sprite.size.y * 0.5) * tf.scale.y.abs();
                let e_min_x = tf.position.x - hw;
                let e_max_x = tf.position.x + hw;
                let e_min_y = tf.position.y - hh;
                let e_max_y = tf.position.y + hh;

                if e_max_x >= min_x && e_min_x <= max_x && e_max_y >= min_y && e_min_y <= max_y {
                    selected.push(entity_id);
                }
            } else if tf.position.x >= min_x
                && tf.position.x <= max_x
                && tf.position.y >= min_y
                && tf.position.y <= max_y
            {
                selected.push(entity_id);
            }
        }
    }

    selected
}

/// Determines which gizmo handle (if any) is hovered/clicked at `world_pos` for an entity transform.
pub fn hit_test_gizmo(
    gizmo_mode: GizmoMode,
    entity_transform: &Transform2D,
    world_pos: Vec2,
    camera_zoom: f32,
) -> Option<GizmoDragAxis> {
    let handle_size = 60.0 / camera_zoom.max(0.1);
    let hit_radius = 16.0 / camera_zoom.max(0.1);
    let center = entity_transform.position;

    match gizmo_mode {
        GizmoMode::Translate => {
            // Center square handle (Both axes)
            if (world_pos.x - center.x).abs() <= hit_radius
                && (world_pos.y - center.y).abs() <= hit_radius
            {
                return Some(GizmoDragAxis::Both);
            }

            // X-axis arrow: from center to center + (handle_size, 0)
            let x_arrow_tip = Vec2::new(center.x + handle_size, center.y);
            if world_pos.x >= center.x
                && world_pos.x <= x_arrow_tip.x + hit_radius
                && (world_pos.y - center.y).abs() <= hit_radius
            {
                return Some(GizmoDragAxis::X);
            }

            // Y-axis arrow: from center to center + (0, handle_size)
            let y_arrow_tip = Vec2::new(center.x, center.y + handle_size);
            if world_pos.y >= center.y
                && world_pos.y <= y_arrow_tip.y + hit_radius
                && (world_pos.x - center.x).abs() <= hit_radius
            {
                return Some(GizmoDragAxis::Y);
            }
        }
        GizmoMode::Rotate => {
            let radius = handle_size * 0.8;
            let dist = ((world_pos.x - center.x).powi(2) + (world_pos.y - center.y).powi(2)).sqrt();
            if (dist - radius).abs() <= hit_radius {
                return Some(GizmoDragAxis::Rotation);
            }
        }
        GizmoMode::Scale => {
            // Center scale
            if (world_pos.x - center.x).abs() <= hit_radius
                && (world_pos.y - center.y).abs() <= hit_radius
            {
                return Some(GizmoDragAxis::Both);
            }
            // X scale box
            let x_box = Vec2::new(center.x + handle_size, center.y);
            if (world_pos.x - x_box.x).abs() <= hit_radius
                && (world_pos.y - x_box.y).abs() <= hit_radius
            {
                return Some(GizmoDragAxis::X);
            }
            // Y scale box
            let y_box = Vec2::new(center.x, center.y + handle_size);
            if (world_pos.x - y_box.x).abs() <= hit_radius
                && (world_pos.y - y_box.y).abs() <= hit_radius
            {
                return Some(GizmoDragAxis::Y);
            }
        }
    }

    None
}

/// Updates editor interactions per frame (camera pan/zoom, selection, gizmo drag).
pub fn update_editor_interactions(world: &mut World, viewport_w: f32, viewport_h: f32) {
    let input = match world.get_resource::<InputState>() {
        Some(i) => i.clone(),
        None => return,
    };

    let mut editor = match world.remove_resource::<EditorState>() {
        Some(e) => e,
        None => return,
    };

    // If currently Playing (unpaused), editor interactions are disabled; input drives the game.
    if editor.play_state == EditorPlayState::Playing {
        world.insert_resource(editor);
        return;
    }

    let mouse_screen = input.mouse_position();
    let mouse_world = editor.screen_to_world(mouse_screen, viewport_w, viewport_h);

    // 1. Gizmo mode shortcuts: W (Move), E (Rotate), R (Scale)
    if input.is_key_just_pressed(KeyCode::KeyW) {
        editor.gizmo_mode = GizmoMode::Translate;
    } else if input.is_key_just_pressed(KeyCode::KeyE) {
        editor.gizmo_mode = GizmoMode::Rotate;
    } else if input.is_key_just_pressed(KeyCode::KeyR) {
        editor.gizmo_mode = GizmoMode::Scale;
    }

    // 2. Camera focus shortcut: F
    if input.is_key_just_pressed(KeyCode::KeyF) {
        if let Some(first_id) = editor.selected_entities.first().cloned() {
            if let Some(entity) = forge_core::find_entity_by_id(world, &first_id) {
                if let Some(tf) = world.get::<Transform2D>(entity) {
                    editor.camera_position = tf.position;
                }
            }
        }
    }

    // 3. Camera pan: Middle mouse or Space + Left mouse
    let is_space = input.is_key_pressed(KeyCode::Space);
    let pan_btn_pressed = input.is_mouse_button_pressed(MouseButton::Middle)
        || (is_space && input.is_mouse_button_pressed(MouseButton::Left));

    if pan_btn_pressed {
        if !editor.is_panning {
            editor.is_panning = true;
            editor.last_pan_mouse_pos = mouse_screen;
        } else {
            let delta_screen = Vec2::new(
                mouse_screen.x - editor.last_pan_mouse_pos.x,
                mouse_screen.y - editor.last_pan_mouse_pos.y,
            );
            editor.last_pan_mouse_pos = mouse_screen;
            let zoom = editor.camera_zoom.max(0.001);
            // Invert screen Y to world Y
            editor.camera_position.x -= delta_screen.x / zoom;
            editor.camera_position.y += delta_screen.y / zoom;
        }
    } else {
        editor.is_panning = false;
    }

    // 4. Camera zoom: Wheel towards cursor
    let wheel = input.mouse_wheel();
    if wheel.y.abs() > 0.001 {
        let zoom_factor = if wheel.y > 0.0 { 1.15 } else { 1.0 / 1.15 };
        let pre_zoom_world = editor.screen_to_world(mouse_screen, viewport_w, viewport_h);
        editor.camera_zoom = (editor.camera_zoom * zoom_factor).clamp(0.05, 50.0);
        let post_zoom_world = editor.screen_to_world(mouse_screen, viewport_w, viewport_h);

        // Adjust camera position so world point under cursor remains invariant
        editor.camera_position.x += pre_zoom_world.x - post_zoom_world.x;
        editor.camera_position.y += pre_zoom_world.y - post_zoom_world.y;
    }

    // 5. Gizmo drag & Selection interactions (Left Mouse button)
    let left_just_pressed = input.is_mouse_button_just_pressed(MouseButton::Left);
    let left_pressed = input.is_mouse_button_pressed(MouseButton::Left);
    let left_just_released = input.is_mouse_button_just_released(MouseButton::Left);
    let shift_held =
        input.is_key_pressed(KeyCode::ShiftLeft) || input.is_key_pressed(KeyCode::ShiftRight);

    if !is_space {
        // Start interaction on left click
        if left_just_pressed {
            let mut hit_gizmo_axis = None;
            let primary_entity = editor.selected_entities.first().cloned();

            if let Some(ref entity_id) = primary_entity {
                if let Some(entity) = forge_core::find_entity_by_id(world, entity_id) {
                    if let Some(tf) = world.get::<Transform2D>(entity) {
                        hit_gizmo_axis =
                            hit_test_gizmo(editor.gizmo_mode, tf, mouse_world, editor.camera_zoom);
                    }
                }
            }

            if let Some(axis) = hit_gizmo_axis {
                // Begin gizmo drag
                let mut original_transforms = Vec::new();
                for id in &editor.selected_entities {
                    if let Some(e) = forge_core::find_entity_by_id(world, id) {
                        if let Some(tf) = world.get::<Transform2D>(e) {
                            original_transforms.push((id.clone(), *tf));
                        }
                    }
                }

                let start_pos = original_transforms
                    .first()
                    .map(|(_, tf)| tf.position)
                    .unwrap_or(Vec2::ZERO);
                editor.active_gizmo_drag = Some(GizmoDragState {
                    axis,
                    start_world_pos: start_pos,
                    start_mouse_world: mouse_world,
                    original_transforms,
                });
            } else {
                // Not hitting gizmo: perform picking or start box select
                let hits = pick_entities_at(world, mouse_world);
                if let Some(topmost) = hits.first() {
                    let hit_id = topmost.entity_id.clone();
                    if shift_held {
                        if editor.selected_entities.contains(&hit_id) {
                            editor.selected_entities.retain(|id| id != &hit_id);
                        } else {
                            editor.selected_entities.push(hit_id);
                        }
                    } else {
                        editor.selected_entities = vec![hit_id];
                    }
                    editor.box_select_start = None;
                    editor.box_select_rect = None;
                } else {
                    // Clicked empty space
                    if !shift_held {
                        editor.selected_entities.clear();
                    }
                    editor.box_select_start = Some(mouse_world);
                    editor.box_select_rect =
                        Some([mouse_world.x, mouse_world.y, mouse_world.x, mouse_world.y]);
                }
            }
        }

        // Active drag update
        if left_pressed {
            if let Some(ref drag) = editor.active_gizmo_drag.clone() {
                let delta = Vec2::new(
                    mouse_world.x - drag.start_mouse_world.x,
                    mouse_world.y - drag.start_mouse_world.y,
                );

                for (id, orig_tf) in &drag.original_transforms {
                    if let Some(e) = forge_core::find_entity_by_id(world, id) {
                        if let Some(mut tf) = world.get_mut::<Transform2D>(e) {
                            match editor.gizmo_mode {
                                GizmoMode::Translate => {
                                    let mut new_pos = orig_tf.position;
                                    match drag.axis {
                                        GizmoDragAxis::X => new_pos.x += delta.x,
                                        GizmoDragAxis::Y => new_pos.y += delta.y,
                                        GizmoDragAxis::Both => {
                                            new_pos.x += delta.x;
                                            new_pos.y += delta.y;
                                        }
                                        _ => {}
                                    }
                                    tf.position = editor.snap_position(new_pos);
                                }
                                GizmoMode::Rotate => {
                                    let center = drag.start_world_pos;
                                    let initial_angle = (drag.start_mouse_world.y - center.y)
                                        .atan2(drag.start_mouse_world.x - center.x);
                                    let current_angle =
                                        (mouse_world.y - center.y).atan2(mouse_world.x - center.x);
                                    let angle_delta = current_angle - initial_angle;
                                    tf.rotation =
                                        editor.snap_rotation(orig_tf.rotation + angle_delta);
                                }
                                GizmoMode::Scale => {
                                    let mut new_scale = orig_tf.scale;
                                    let factor = 1.0 + (delta.x + delta.y) * 0.01;
                                    match drag.axis {
                                        GizmoDragAxis::X => {
                                            new_scale.x =
                                                (orig_tf.scale.x * (1.0 + delta.x * 0.01)).max(0.01)
                                        }
                                        GizmoDragAxis::Y => {
                                            new_scale.y =
                                                (orig_tf.scale.y * (1.0 + delta.y * 0.01)).max(0.01)
                                        }
                                        GizmoDragAxis::Both => {
                                            new_scale.x = (orig_tf.scale.x * factor).max(0.01);
                                            new_scale.y = (orig_tf.scale.y * factor).max(0.01);
                                        }
                                        _ => {}
                                    }
                                    tf.scale = new_scale;
                                }
                            }
                        }
                    }
                }
            } else if let Some(start) = editor.box_select_start {
                // Update box select rectangle
                editor.box_select_rect = Some([start.x, start.y, mouse_world.x, mouse_world.y]);
            }
        }

        // Release: finalize gizmo command or box selection
        if left_just_released {
            if let Some(drag) = editor.active_gizmo_drag.take() {
                // Record undo command for modified transforms
                for (id, orig_tf) in drag.original_transforms {
                    if let Some(e) = forge_core::find_entity_by_id(world, &id) {
                        if let Some(curr_tf) = world.get::<Transform2D>(e) {
                            if *curr_tf != orig_tf {
                                let mut prev_map = std::collections::HashMap::new();
                                prev_map.insert(
                                    "position".to_string(),
                                    serde_json::json!([orig_tf.position.x, orig_tf.position.y])
                                        .to_string(),
                                );
                                prev_map
                                    .insert("rotation".to_string(), orig_tf.rotation.to_string());
                                prev_map.insert(
                                    "scale".to_string(),
                                    serde_json::json!([orig_tf.scale.x, orig_tf.scale.y])
                                        .to_string(),
                                );

                                let mut new_map = std::collections::HashMap::new();
                                new_map.insert(
                                    "position".to_string(),
                                    serde_json::json!([curr_tf.position.x, curr_tf.position.y])
                                        .to_string(),
                                );
                                new_map
                                    .insert("rotation".to_string(), curr_tf.rotation.to_string());
                                new_map.insert(
                                    "scale".to_string(),
                                    serde_json::json!([curr_tf.scale.x, curr_tf.scale.y])
                                        .to_string(),
                                );

                                let cmd = SceneCommand::SetComponent {
                                    entity_id: id.clone(),
                                    component_name: "Transform2D".to_string(),
                                    new_data: new_map,
                                    prev_data: Some(prev_map),
                                };

                                if let Some(mut stack) = world.get_resource_mut::<CommandStack>() {
                                    stack.record_executed(cmd, CommandAuthor::User);
                                }
                            }
                        }
                    }
                }
            } else if let Some(rect) = editor.box_select_rect.take() {
                editor.box_select_start = None;
                let boxed_ids = pick_entities_in_rect(world, rect);
                if shift_held {
                    for id in boxed_ids {
                        if !editor.selected_entities.contains(&id) {
                            editor.selected_entities.push(id);
                        }
                    }
                } else if !boxed_ids.is_empty() {
                    editor.selected_entities = boxed_ids;
                }
            }
        }
    }

    world.insert_resource(editor);
}
