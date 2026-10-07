//! 2D Sprite Renderer orchestrating draw calls, batching, and pipeline execution.

use crate::batch::SpriteBatcher;
use crate::camera::CameraUniform;
use crate::context::RenderContext;
use crate::error::RenderError;
use crate::target::OffscreenTarget;
use crate::vertex::SpriteVertex;
use forge_core::{Sprite, Transform2D};
use std::sync::Arc;

const INITIAL_SPRITE_CAPACITY: usize = 256;

/// 2D Sprite Renderer executing batched quad rendering using WGPU.
pub struct Renderer2D {
    /// Shared GPU context.
    pub context: Arc<RenderContext>,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_buffer: wgpu::Buffer,
    index_capacity: usize,
    batcher: SpriteBatcher,
}

impl Renderer2D {
    /// Creates a new 2D renderer using the provided GPU context.
    pub fn new(context: Arc<RenderContext>) -> Self {
        let initial_uniform = CameraUniform::default();
        let camera_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera_uniform_buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        context
            .queue
            .write_buffer(&camera_buffer, 0, bytemuck::bytes_of(&initial_uniform));

        let camera_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera_bind_group"),
                layout: &context.camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

        let vertex_capacity = INITIAL_SPRITE_CAPACITY * 4;
        let vertex_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprite_vertex_buffer"),
            size: (vertex_capacity * std::mem::size_of::<SpriteVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_capacity = INITIAL_SPRITE_CAPACITY * 6;
        let index_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprite_index_buffer"),
            size: (index_capacity * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            context,
            camera_buffer,
            camera_bind_group,
            vertex_buffer,
            vertex_capacity,
            index_buffer,
            index_capacity,
            batcher: SpriteBatcher::new(),
        }
    }

    /// Queues a sprite to be drawn in the next frame.
    pub fn draw_sprite(
        &mut self,
        transform: Transform2D,
        sprite: Sprite,
        texture_bind_group: Arc<wgpu::BindGroup>,
    ) {
        self.batcher.push(transform, sprite, texture_bind_group);
    }

    /// Clears any pending queued sprites without drawing.
    pub fn clear_queue(&mut self) {
        self.batcher.clear();
    }

    /// Renders queued sprites to an offscreen target.
    pub fn render_to_offscreen(
        &mut self,
        target: &OffscreenTarget,
        clear_color: Option<[f64; 4]>,
        camera_uniform: &CameraUniform,
    ) -> Result<(), RenderError> {
        self.render_to_view(&target.view, target.format, clear_color, camera_uniform)
    }

    /// Renders queued sprites to any target texture view.
    pub fn render_to_view(
        &mut self,
        target_view: &wgpu::TextureView,
        target_format: wgpu::TextureFormat,
        clear_color: Option<[f64; 4]>,
        camera_uniform: &CameraUniform,
    ) -> Result<(), RenderError> {
        // Update camera uniform
        self.context
            .queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(camera_uniform));

        // Build sorted geometry and batches
        let (vertices, indices, batches) = self.batcher.build();

        // Ensure vertex buffer capacity
        if vertices.len() > self.vertex_capacity {
            self.vertex_capacity = vertices.len().max(self.vertex_capacity * 2);
            self.vertex_buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("sprite_vertex_buffer_resized"),
                size: (self.vertex_capacity * std::mem::size_of::<SpriteVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Ensure index buffer capacity
        if indices.len() > self.index_capacity {
            self.index_capacity = indices.len().max(self.index_capacity * 2);
            self.index_buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("sprite_index_buffer_resized"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Upload vertex & index data
        if !vertices.is_empty() {
            let vertex_bytes = bytemuck::cast_slice(&vertices);
            self.context
                .queue
                .write_buffer(&self.vertex_buffer, 0, vertex_bytes);

            let index_bytes = bytemuck::cast_slice(&indices);
            self.context
                .queue
                .write_buffer(&self.index_buffer, 0, index_bytes);
        }

        let pipeline = self.context.get_or_create_pipeline(target_format);

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("renderer2d_command_encoder"),
                });

        let load_op = match clear_color {
            Some(c) => wgpu::LoadOp::Clear(wgpu::Color {
                r: c[0],
                g: c[1],
                b: c[2],
                a: c[3],
            }),
            None => wgpu::LoadOp::Load,
        };

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("renderer2d_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !batches.is_empty() {
                render_pass.set_pipeline(&pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

                let vertex_slice_end =
                    (vertices.len() * std::mem::size_of::<SpriteVertex>()) as u64;
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..vertex_slice_end));

                let index_slice_end = (indices.len() * std::mem::size_of::<u32>()) as u64;
                render_pass.set_index_buffer(
                    self.index_buffer.slice(..index_slice_end),
                    wgpu::IndexFormat::Uint32,
                );

                for batch in &batches {
                    render_pass.set_bind_group(1, batch.bind_group.as_ref(), &[]);
                    render_pass.draw_indexed(
                        batch.index_start..batch.index_start + batch.index_count,
                        0,
                        0..1,
                    );
                }
            }
        }

        self.context.queue.submit(Some(encoder.finish()));
        self.batcher.clear();

        Ok(())
    }
}
