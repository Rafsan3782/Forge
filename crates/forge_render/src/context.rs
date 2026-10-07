//! Render context managing WGPU instance, adapter, device, queue, and pipeline caching.

use crate::camera::CameraUniform;
use crate::error::RenderError;
use crate::texture::{create_default_sampler, create_texture_bind_group_layout};
use crate::vertex::SpriteVertex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

static ADAPTER_LOGGED: AtomicBool = AtomicBool::new(false);

/// Options for initializing the GPU render context.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Forces the use of a software/fallback adapter (e.g. WARP on Windows).
    pub force_fallback_adapter: bool,
    /// Backends to scan for adapters.
    pub backends: wgpu::Backends,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            force_fallback_adapter: false,
            backends: wgpu::Backends::all(),
        }
    }
}

/// Core GPU context holding active WGPU handles, layouts, and cached render pipelines.
pub struct RenderContext {
    /// WGPU instance.
    pub instance: wgpu::Instance,
    /// WGPU adapter.
    pub adapter: wgpu::Adapter,
    /// Logical GPU device.
    pub device: Arc<wgpu::Device>,
    /// GPU command queue.
    pub queue: Arc<wgpu::Queue>,
    /// Camera uniform bind group layout.
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    /// Texture and sampler bind group layout.
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    /// Default linear texture sampler.
    pub default_sampler: wgpu::Sampler,
    /// Compiled WGSL shader module.
    pub shader_module: wgpu::ShaderModule,
    /// Pipeline layout combining camera and texture bind groups.
    pub pipeline_layout: wgpu::PipelineLayout,
    /// Adapter metadata information.
    pub adapter_info: wgpu::AdapterInfo,
    /// Cached pipelines for different render target texture formats.
    pipelines: Mutex<HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>>,
}

impl RenderContext {
    /// Initializes the render context asynchronously with optional compatible surface.
    pub async fn new_with_surface_async(
        options: RenderOptions,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: options.backends,
            ..Default::default()
        });

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface,
                force_fallback_adapter: options.force_fallback_adapter,
            })
            .await
        {
            Some(a) => Some(a),
            None if !options.force_fallback_adapter => {
                tracing::warn!("Primary GPU adapter request returned None; attempting fallback software adapter...");
                instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::None,
                        compatible_surface: surface,
                        force_fallback_adapter: true,
                    })
                    .await
            }
            None => None,
        };

        let adapter = match adapter {
            Some(a) => a,
            None => return Err(RenderError::NoAdapterFound),
        };

        let adapter_info = adapter.get_info();
        if !ADAPTER_LOGGED.swap(true, Ordering::SeqCst) {
            tracing::info!(
                "GPU Adapter initialized: name='{}', vendor={:#x}, device={:#x}, type={:?}, backend={:?}, driver='{}'",
                adapter_info.name,
                adapter_info.vendor,
                adapter_info.device,
                adapter_info.device_type,
                adapter_info.backend,
                adapter_info.driver
            );
        }

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("forge_render_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| RenderError::RequestDeviceFailed(e.to_string()))?;

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<CameraUniform>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout = create_texture_bind_group_layout(&device);
        let default_sampler = create_default_sampler(&device);

        let shader_code = include_str!("shader.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("forge_sprite_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("forge_sprite_pipeline_layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        Ok(Self {
            instance,
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
            camera_bind_group_layout,
            texture_bind_group_layout,
            default_sampler,
            shader_module,
            pipeline_layout,
            adapter_info,
            pipelines: Mutex::new(HashMap::new()),
        })
    }

    /// Synchronous initialization blocking on async setup.
    pub fn new(options: RenderOptions) -> Result<Self, RenderError> {
        pollster::block_on(Self::new_with_surface_async(options, None))
    }

    /// Synchronous initialization with a target window surface.
    pub fn new_with_surface(
        options: RenderOptions,
        surface: &wgpu::Surface<'_>,
    ) -> Result<Self, RenderError> {
        pollster::block_on(Self::new_with_surface_async(options, Some(surface)))
    }

    /// Configures a window surface with optimal sRGB format and dimensions.
    pub fn configure_surface(
        &self,
        surface: &wgpu::Surface<'_>,
        width: u32,
        height: u32,
    ) -> wgpu::SurfaceConfiguration {
        let caps = surface.get_capabilities(&self.adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&self.device, &config);
        config
    }

    /// Gets or creates a pipeline for the given render target texture format.
    pub fn get_or_create_pipeline(&self, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let mut cache = self.pipelines.lock().unwrap();
        if let Some(pipeline) = cache.get(&format) {
            return pipeline.clone();
        }

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("sprite_pipeline_{:?}", format)),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader_module,
                    entry_point: Some("vs_main"),
                    buffers: &[SpriteVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader_module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        cache.insert(format, pipeline.clone());
        pipeline
    }
}
