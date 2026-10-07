//! GPU texture representations and texture bind group utilities.

use crate::error::RenderError;
use std::sync::Arc;

/// GPU texture containing the underlying WGPU texture, view, size, and bind group.
#[derive(Debug, Clone)]
pub struct GpuTexture {
    /// The underlying GPU texture.
    pub texture: Arc<wgpu::Texture>,
    /// Default view for sampling.
    pub view: Arc<wgpu::TextureView>,
    /// Bind group binding texture view and sampler for shaders.
    pub bind_group: Arc<wgpu::BindGroup>,
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
}

impl GpuTexture {
    /// Creates a new GPU texture from raw RGBA8 bytes.
    #[allow(clippy::too_many_arguments)]
    pub fn from_rgba8(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        rgba: &[u8],
        width: u32,
        height: u32,
        label: Option<&str>,
    ) -> Result<Self, RenderError> {
        if width == 0 || height == 0 || rgba.len() != (width * height * 4) as usize {
            return Err(RenderError::InvalidDimensions { width, height });
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label,
        });

        Ok(Self {
            texture: Arc::new(texture),
            view: Arc::new(view),
            bind_group: Arc::new(bind_group),
            width,
            height,
        })
    }

    /// Creates a 2x2 solid magenta (RGBA: 255, 0, 255, 255) placeholder texture.
    pub fn create_magenta_placeholder(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> Result<Self, RenderError> {
        let magenta = [
            255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255,
        ];
        Self::from_rgba8(
            device,
            queue,
            layout,
            sampler,
            &magenta,
            2,
            2,
            Some("magenta_placeholder_texture"),
        )
    }
}

/// Creates the standard texture bind group layout (texture view at binding 0, sampler at binding 1).
pub fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("texture_bind_group_layout"),
        entries: &[
            // Texture view
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// Creates a default linear sampler with clamp-to-edge addressing.
pub fn create_default_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("default_linear_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
