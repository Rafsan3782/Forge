//! Render targets for offscreen texture rendering and window surfaces.

use crate::error::RenderError;
use std::sync::mpsc;

/// Offscreen render target that can be rendered to and read back to CPU memory.
pub struct OffscreenTarget {
    /// Render target texture.
    pub texture: wgpu::Texture,
    /// Texture view used as a color attachment.
    pub view: wgpu::TextureView,
    /// Format of the offscreen texture.
    pub format: wgpu::TextureFormat,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl OffscreenTarget {
    /// Creates a new offscreen render target with format `Rgba8UnormSrgb`.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Result<Self, RenderError> {
        Self::with_format(device, width, height, wgpu::TextureFormat::Rgba8UnormSrgb)
    }

    /// Creates a new offscreen render target with the specified format.
    pub fn with_format(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidDimensions { width, height });
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen_target_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("offscreen_target_view"),
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            format,
            width,
            height,
        })
    }

    /// Reads back RGBA8 pixels from the offscreen texture to CPU memory.
    pub fn read_rgba(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u8>, RenderError> {
        // WGPU requires bytes_per_row to be aligned to 256 bytes
        let bytes_per_row = (self.width * 4 + 255) & !255;
        let buffer_size = (bytes_per_row * self.height) as wgpu::BufferAddress;

        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("offscreen_readback_buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("offscreen_readback_encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        let slice = readback_buffer.slice(..);
        let (tx, rx) = mpsc::channel();

        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });

        device.poll(wgpu::Maintain::Wait);

        rx.recv()
            .map_err(|e| RenderError::BufferReadError(e.to_string()))?
            .map_err(|e| RenderError::BufferReadError(e.to_string()))?;

        let mapped = slice.get_mapped_range();
        let mut unpadded = Vec::with_capacity((self.width * self.height * 4) as usize);

        for row in 0..self.height {
            let start = (row * bytes_per_row) as usize;
            let end = start + (self.width * 4) as usize;
            unpadded.extend_from_slice(&mapped[start..end]);
        }

        drop(mapped);
        readback_buffer.unmap();

        Ok(unpadded)
    }
}
