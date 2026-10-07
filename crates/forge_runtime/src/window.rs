//! Interactive window runner using Winit and WGPU surface rendering.

use forge_assets::TextureStore;
use forge_core::{GameLoop, Scene, World};
use forge_render::{RenderContext, RenderOptions, Renderer2D};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::cli::CliArgs;
use crate::render_scene::{render_world_scene, DEFAULT_CLEAR_COLOR};

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
}

impl WindowApp {
    /// Initializes a new WindowApp from CLI arguments, loading the initial scene into the ECS world.
    pub fn new(args: CliArgs) -> Result<Self, Box<dyn std::error::Error>> {
        let scene = Scene::load_from_file(&args.scene)?;
        let mut world = World::new();
        scene.spawn_into_world(&mut world)?;

        Ok(Self {
            args,
            world,
            game_loop: GameLoop::new(),
            state: None,
            last_frame_instant: None,
        })
    }

    fn init_window_state(
        event_loop: &ActiveEventLoop,
        args: &CliArgs,
    ) -> Result<WindowState, Box<dyn std::error::Error>> {
        let window_attrs = Window::default_attributes()
            .with_title("Forge Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(args.width, args.height));

        let window = Arc::new(event_loop.create_window(window_attrs)?);

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
                Ok(state) => {
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
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        match event {
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
                let now = Instant::now();
                let delta = match self.last_frame_instant {
                    Some(prev) => now.duration_since(prev),
                    None => Duration::from_secs_f64(1.0 / 60.0),
                };
                self.last_frame_instant = Some(now);

                // Step simulation through fixed timestep loop
                self.game_loop.step(&mut self.world, delta);

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
