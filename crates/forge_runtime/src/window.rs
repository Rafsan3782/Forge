//! Interactive window runner using Winit and WGPU surface rendering.

use forge_assets::TextureStore;
use forge_core::{GameLoop, Scene, World};
use forge_input::{
    fixed_input_tick_system, pump_winit_event, ActionMap, FixedInput, GilrsAdapter,
    InputRingBuffer, InputState,
};
use forge_render::{RenderContext, RenderOptions, Renderer2D};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::bridge_ops::process_bridge_commands;
use crate::cli::CliArgs;
use crate::demo_input::{demo_character_controller_system, demo_hero_movement_system};
use crate::render_scene::{render_world_scene, DEFAULT_CLEAR_COLOR};
use forge_bridge::{create_bridge_channel, BridgeCommandReceiver, BridgeServer};

struct WindowState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    context: Arc<RenderContext>,
    renderer: Renderer2D,
    texture_store: TextureStore,
}

/// Winit ApplicationHandler implementing event-driven window lifecycle and render loop.
pub struct WindowApp {
    args: CliArgs,
    world: World,
    game_loop: GameLoop,
    state: Option<WindowState>,
    last_frame_instant: Option<Instant>,
    command_rx: Option<BridgeCommandReceiver>,
    _bridge_server: Option<BridgeServer>,
    gilrs: GilrsAdapter,
    watcher: Option<forge_assets::HotReloadWatcher>,
}

impl WindowApp {
    /// Initializes a new WindowApp from CLI arguments, loading the initial scene into the ECS world.
    pub fn new(args: CliArgs) -> Result<Self, Box<dyn std::error::Error>> {
        let scene_path = Path::new(&args.scene);
        let scene_dir = scene_path.parent().unwrap_or(Path::new("."));
        let assets_dir = scene_dir.join("assets");

        if args.scene.contains("tilemap-demo")
            && (!assets_dir.exists() || !assets_dir.join("tileset.png").exists())
        {
            let _ = crate::procedural::generate_tilemap_assets(&assets_dir);
        } else if args.scene.contains("hello")
            && (!assets_dir.exists() || !assets_dir.join("hero.png").exists())
        {
            let _ = crate::procedural::generate_hello_assets(&assets_dir);
        } else if args.scene.contains("audio-demo")
            && (!assets_dir.exists() || !assets_dir.join("tone1.wav").exists())
        {
            let _ = crate::procedural::generate_audio_demo_assets(&assets_dir);
        }

        // Initialize and scan AssetDatabase
        let mut asset_db = forge_assets::AssetDatabase::new(&assets_dir);
        if let Err(e) = asset_db.scan_and_index() {
            tracing::warn!("Failed to scan asset database at {:?}: {}", assets_dir, e);
        }

        let mut animation_store = forge_assets::AnimationStore::new();
        animation_store.set_base_dir(&assets_dir);
        let mut sheet_store = forge_assets::SpriteSheetStore::new();
        sheet_store.set_base_dir(&assets_dir);
        let mut tileset_store = forge_assets::TilesetStore::new();
        tileset_store.set_base_dir(&assets_dir);

        let scene = Scene::load_from_file(&args.scene)?;
        let mut world = World::new();
        scene.spawn_into_world(&mut world)?;

        // Insert asset resources
        world.insert_resource(asset_db);
        world.insert_resource(animation_store);
        world.insert_resource(sheet_store);
        world.insert_resource(tileset_store);
        world.insert_resource(forge_core::EventBus::new());

        // Initialize ActionMap
        let action_map = if let Some(ref map_path) = args.input_map {
            ActionMap::load_from_file(map_path)?
        } else {
            let candidate = scene_dir.join("input_map.json");
            if candidate.exists() {
                ActionMap::load_from_file(&candidate)?
            } else {
                ActionMap::default()
            }
        };

        // Insert input resources
        world.insert_resource(InputState::default());
        world.insert_resource(action_map);
        world.insert_resource(FixedInput::default());
        world.insert_resource(InputRingBuffer::new(2048));

        // Insert physics resources
        world.insert_resource(forge_physics::PhysicsWorld2D::default());
        world.insert_resource(crate::render_scene::PhysicsDebugDraw::default());

        // Insert audio resource
        let audio_server = forge_audio::AudioServer::new(&assets_dir);
        world.insert_resource(audio_server);

        // Insert script server if runtime initialized
        if let Ok(script_server) =
            forge_script::ScriptServer::new(forge_script::SandboxConfig::default())
        {
            world.insert_resource(script_server);
        }

        // Insert CommandStack resource for undo/redo and scene history
        let mut command_stack = forge_core::CommandStack::new();
        command_stack.mark_clean(Some(args.scene.clone()));
        world.insert_resource(command_stack);

        // Insert EditorState resource for editor camera, picking, gizmos, and play mode
        world.insert_resource(crate::editor::EditorState::new());

        let mut game_loop = GameLoop::new();
        game_loop.add_pre_update_system(fixed_input_tick_system);
        game_loop.add_pre_update_system(forge_script::script_prepare_frame_system);
        game_loop.add_pre_update_system(forge_physics::physics_before_step_system);
        game_loop.add_pre_update_system(forge_script::script_start_system);
        game_loop.add_update_system(forge_physics::physics_step_system);
        game_loop.add_update_system(forge_script::script_update_system);
        game_loop.add_update_system(forge_script::script_apply_commands_system);
        game_loop.add_post_update_system(forge_physics::physics_after_step_system);
        game_loop.add_update_system(forge_assets::animator_update_system);
        game_loop.add_update_system(forge_audio::audio_source_start_system);
        game_loop.add_post_update_system(forge_audio::audio_spatial_update_system);
        game_loop.add_update_system(crate::demo_input::demo_audio_keyboard_system);

        game_loop.add_update_system(demo_character_controller_system);

        if args.demo_input {
            game_loop.add_update_system(demo_hero_movement_system);
        }

        let watcher = if assets_dir.exists() {
            forge_assets::HotReloadWatcher::new(&assets_dir).ok()
        } else {
            None
        };

        let (command_rx, _bridge_server) = if !args.no_bridge {
            let (cmd_tx, cmd_rx) = create_bridge_channel();
            let server = BridgeServer::start_background(args.bridge_token.clone(), cmd_tx)?;
            (Some(cmd_rx), Some(server))
        } else {
            (None, None)
        };

        Ok(Self {
            args,
            world,
            game_loop,
            state: None,
            last_frame_instant: None,
            command_rx,
            _bridge_server,
            gilrs: GilrsAdapter::new(),
            watcher,
        })
    }

    fn init_window_state(
        event_loop: &ActiveEventLoop,
        args: &CliArgs,
    ) -> Result<WindowState, Box<dyn std::error::Error>> {
        let mut window_attrs = Window::default_attributes()
            .with_title("Forge Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(args.width, args.height));

        #[cfg(windows)]
        if let Some(parent_str) = &args.embed_parent {
            let parent_hwnd: Result<isize, _> = if let Some(stripped) = parent_str
                .strip_prefix("0x")
                .or_else(|| parent_str.strip_prefix("0X"))
            {
                isize::from_str_radix(stripped, 16)
            } else {
                parent_str.parse::<isize>()
            };

            if let Ok(hwnd) = parent_hwnd {
                if let Some(non_zero) = std::num::NonZeroIsize::new(hwnd) {
                    use winit::raw_window_handle::{RawWindowHandle, Win32WindowHandle};
                    let handle = Win32WindowHandle::new(non_zero);
                    let rwh = RawWindowHandle::Win32(handle);
                    window_attrs = unsafe { window_attrs.with_parent_window(Some(rwh)) };
                    window_attrs = window_attrs.with_decorations(false);
                }
            }
        }

        let window = Arc::new(event_loop.create_window(window_attrs)?);

        #[cfg(windows)]
        {
            use std::io::Write;
            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(wh) = window.window_handle() {
                if let RawWindowHandle::Win32(h) = wh.as_raw() {
                    let child_hwnd = h.hwnd.get();
                    println!("FORGE_WINDOW_READY {{\"hwnd\":{}}}", child_hwnd);
                    let _ = std::io::stdout().flush();
                }
            }
        }

        let render_options = RenderOptions {
            force_fallback_adapter: args.force_fallback,
            backends: wgpu::Backends::all(),
        };

        // Create RenderContext and Window Surface from the exact same WGPU instance
        let (context, surface) =
            RenderContext::new_with_surface_target(render_options, window.clone())?;
        let context = Arc::new(context);

        let size = window.inner_size();
        let surface_config = context.configure_surface(&surface, size.width, size.height);

        let mut texture_store = TextureStore::new(&context)?;
        texture_store.set_base_dir(Path::new("."));

        let renderer = Renderer2D::new(context.clone());

        Ok(WindowState {
            window,
            surface,
            surface_config,
            context,
            renderer,
            texture_store,
        })
    }
}

impl ApplicationHandler for WindowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            match Self::init_window_state(event_loop, &self.args) {
                Ok(mut state) => {
                    if let Some(db) = self.world.get_resource::<forge_assets::AssetDatabase>() {
                        state.texture_store.set_database(db.clone());
                    }
                    self.state = Some(state);
                }
                Err(e) => {
                    tracing::error!("Failed to initialize window and GPU surface: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Forward window event to input system
        if let Some(mut input_state) = self.world.get_resource_mut::<InputState>() {
            pump_winit_event(&event, &mut input_state, None);
        }

        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F1),
                        state: winit::event::ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                if let Some(mut pdd) = self
                    .world
                    .get_resource_mut::<crate::render_scene::PhysicsDebugDraw>()
                {
                    pdd.toggle();
                    tracing::info!("Toggled physics debug draw: {}", pdd.enabled);
                }
            }
            WindowEvent::CloseRequested => {
                tracing::info!("Window close requested. Exiting cleanly.");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if physical_size.width > 0 && physical_size.height > 0 {
                    state.surface_config.width = physical_size.width;
                    state.surface_config.height = physical_size.height;
                    state
                        .surface
                        .configure(&state.context.device, &state.surface_config);
                    state.window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let size = state.window.inner_size();
                if size.width > 0 && size.height > 0 {
                    state.surface_config.width = size.width;
                    state.surface_config.height = size.height;
                    state
                        .surface
                        .configure(&state.context.device, &state.surface_config);
                    state.window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let mut shutdown_requested = false;
                if let Some(ref mut rx) = self.command_rx {
                    process_bridge_commands(
                        rx,
                        &mut self.world,
                        &mut state.renderer,
                        &mut state.texture_store,
                        &state.context,
                        state.surface_config.width,
                        state.surface_config.height,
                        &mut shutdown_requested,
                    );
                }

                if shutdown_requested {
                    tracing::info!("Shutdown requested via bridge project.stop. Exiting cleanly.");
                    event_loop.exit();
                    return;
                }

                // Poll hot reload events
                if let Some(ref mut w) = self.watcher {
                    for event in w.poll_events() {
                        match event {
                            forge_assets::HotReloadEvent::Texture(p) => {
                                let path_str = p.to_string_lossy();
                                let asset_id = if let Some(db) =
                                    self.world.get_resource::<forge_assets::AssetDatabase>()
                                {
                                    db.get_id_by_path(&p).unwrap_or(&path_str).to_string()
                                } else {
                                    path_str.to_string()
                                };
                                let _ = state.texture_store.reload(&asset_id, &state.context);
                                tracing::info!("Hot reloaded texture: {:?}", p);
                            }
                            forge_assets::HotReloadEvent::SpriteSheet(p) => {
                                if let Some(mut sheets) = self
                                    .world
                                    .get_resource_mut::<forge_assets::SpriteSheetStore>()
                                {
                                    let path_str = p.to_string_lossy();
                                    let _ = sheets.reload(&path_str, &p);
                                    tracing::info!("Hot reloaded spritesheet: {:?}", p);
                                }
                            }
                            forge_assets::HotReloadEvent::Animation(p) => {
                                if let Some(mut anims) = self
                                    .world
                                    .get_resource_mut::<forge_assets::AnimationStore>()
                                {
                                    let path_str = p.to_string_lossy();
                                    let _ = anims.reload(&path_str, &p);
                                    tracing::info!("Hot reloaded animation: {:?}", p);
                                }
                            }
                            forge_assets::HotReloadEvent::Scene(p) => {
                                if let Ok(new_scene) = Scene::load_from_file(&p) {
                                    self.world.clear_entities();
                                    let _ = new_scene.spawn_into_world(&mut self.world);
                                    tracing::info!("Hot reloaded scene from {:?}", p);
                                }
                            }
                            forge_assets::HotReloadEvent::Meta(p) => {
                                if let Some(mut db) =
                                    self.world.get_resource_mut::<forge_assets::AssetDatabase>()
                                {
                                    let _ = db.scan_and_index();
                                    state.texture_store.set_database(db.clone());
                                    tracing::info!("Hot reloaded asset database metadata: {:?}", p);
                                }
                            }
                            forge_assets::HotReloadEvent::Audio(p) => {
                                if let Some(mut audio) =
                                    self.world.get_resource_mut::<forge_audio::AudioServer>()
                                {
                                    audio.invalidate_cache(&p);
                                    tracing::info!("Invalidated audio cache for: {:?}", p);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // Poll gamepad events into InputState
                if let Some(mut input_state) = self.world.get_resource_mut::<InputState>() {
                    self.gilrs.poll_events(&mut input_state, None);
                }

                // Sync InputState + ActionMap -> FixedInput
                forge_input::sync_world_frame_input(&mut self.world);

                let now = Instant::now();
                let delta = match self.last_frame_instant {
                    Some(prev) => now.duration_since(prev),
                    None => Duration::from_secs_f64(1.0 / 60.0),
                };
                self.last_frame_instant = Some(now);

                // Editor interactions (pan, zoom, picking, gizmos)
                crate::editor::update_editor_interactions(
                    &mut self.world,
                    state.surface_config.width as f32,
                    state.surface_config.height as f32,
                );

                // Check whether to step simulation (Edit mode does not run physics/scripts/audio)
                let should_step = if let Some(mut ed) =
                    self.world.get_resource_mut::<crate::editor::EditorState>()
                {
                    match ed.play_state {
                        crate::editor::EditorPlayState::Playing => true,
                        crate::editor::EditorPlayState::Paused => {
                            if ed.step_frames_remaining > 0 {
                                ed.step_frames_remaining -= 1;
                                true
                            } else {
                                false
                            }
                        }
                        crate::editor::EditorPlayState::Edit => false,
                    }
                } else {
                    true
                };

                if should_step {
                    // Step simulation through fixed timestep loop
                    self.game_loop.step(&mut self.world, delta);
                }

                // Drain and log physics trigger and collision events
                if let Some(mut event_bus) = self
                    .world
                    .get_resource_mut::<forge_core::events::EventBus>()
                {
                    for event in event_bus.read::<forge_physics::TriggerEvent>() {
                        match event {
                            forge_physics::TriggerEvent::Enter {
                                trigger_entity,
                                other_entity,
                            } => {
                                tracing::info!(
                                    "Trigger Enter: trigger entity {:?} entered by entity {:?}",
                                    trigger_entity,
                                    other_entity
                                );
                            }
                            forge_physics::TriggerEvent::Exit {
                                trigger_entity,
                                other_entity,
                            } => {
                                tracing::info!(
                                    "Trigger Exit: trigger entity {:?} exited by entity {:?}",
                                    trigger_entity,
                                    other_entity
                                );
                            }
                        }
                    }
                    event_bus.clear::<forge_physics::TriggerEvent>();
                }

                // Advance frame in InputState (clears just_pressed / just_released, counts down holds)
                if let Some(mut input_state) = self.world.get_resource_mut::<InputState>() {
                    input_state.start_new_frame();
                }

                let output_frame = match state.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state
                            .surface
                            .configure(&state.context.device, &state.surface_config);
                        state.window.request_redraw();
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        tracing::error!("Out of GPU memory acquiring surface texture. Exiting.");
                        event_loop.exit();
                        return;
                    }
                    Err(e) => {
                        tracing::warn!("Surface texture error: {:?}", e);
                        return;
                    }
                };

                let view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                if let Err(e) = render_world_scene(
                    &self.world,
                    &mut state.renderer,
                    &mut state.texture_store,
                    &state.context,
                    &view,
                    state.surface_config.format,
                    state.surface_config.width as f32,
                    state.surface_config.height as f32,
                    Some(DEFAULT_CLEAR_COLOR),
                ) {
                    tracing::error!("Failed to render frame: {:?}", e);
                }

                output_frame.present();
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

/// Runs the interactive windowed game loop using Winit.
pub fn run_window(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = WindowApp::new(args)?;
    event_loop.run_app(&mut app)?;
    Ok(())
}
