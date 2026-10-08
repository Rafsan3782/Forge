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

use crate::bridge_ops::process_bridge_commands;
use crate::cli::CliArgs;
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
}

impl WindowApp {
    /// Initializes a new WindowApp from CLI arguments, loading the initial scene into the ECS world.
    pub fn new(args: CliArgs) -> Result<Self, Box<dyn std::error::Error>> {
        let scene = Scene::load_from_file(&args.scene)?;
        let mut world = World::new();
        scene.spawn_into_world(&mut world)?;

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
            game_loop: GameLoop::new(),
            state: None,
            last_frame_instant: None,
            command_rx,
            _bridge_server,
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
