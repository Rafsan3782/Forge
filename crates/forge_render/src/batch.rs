//! Sprite sorting, batching, and index generation.

use crate::vertex::{generate_sprite_quad, SpriteVertex};
use forge_core::{Sprite, Transform2D};
use std::sync::Arc;

/// An individual sprite item queued for rendering.
#[derive(Clone)]
pub struct QueuedSprite {
    /// Spatial transform of the sprite.
    pub transform: Transform2D,
    /// Sprite properties including z-order and tint.
    pub sprite: Sprite,
    /// GPU texture bind group to sample from.
    pub texture_bind_group: Arc<wgpu::BindGroup>,
}

/// A batched draw call sharing a single texture bind group.
#[derive(Debug, Clone)]
pub struct DrawBatch {
    /// The texture bind group used for this draw call.
    pub bind_group: Arc<wgpu::BindGroup>,
    /// Starting index within the index buffer.
    pub index_start: u32,
    /// Number of indices to draw.
    pub index_count: u32,
}

/// Batches sprites by sorting them by z-order and merging contiguous draws with the same texture.
pub struct SpriteBatcher {
    sprites: Vec<QueuedSprite>,
}

impl Default for SpriteBatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl SpriteBatcher {
    /// Creates a new empty sprite batcher.
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
        }
    }

    /// Pushes a sprite into the queue for the current frame.
    pub fn push(
        &mut self,
        transform: Transform2D,
        sprite: Sprite,
        texture_bind_group: Arc<wgpu::BindGroup>,
    ) {
        self.sprites.push(QueuedSprite {
            transform,
            sprite,
            texture_bind_group,
        });
    }

    /// Clears all queued sprites.
    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    /// Builds sorted vertex and index lists along with draw batches.
    ///
    /// Sprites are sorted back-to-front (ascending by `z_order`). Contiguous sprites
    /// with the same texture bind group are combined into a single draw call.
    pub fn build(&mut self) -> (Vec<SpriteVertex>, Vec<u32>, Vec<DrawBatch>) {
        // Sort stably by z-order ascending
        self.sprites
            .sort_by(|a, b| a.sprite.z_order.cmp(&b.sprite.z_order));

        let mut vertices = Vec::with_capacity(self.sprites.len() * 4);
        let mut indices = Vec::with_capacity(self.sprites.len() * 6);
        let mut batches = Vec::<DrawBatch>::new();

        for sprite_item in &self.sprites {
            let base_vertex = vertices.len() as u32;
            let quad = generate_sprite_quad(&sprite_item.transform, &sprite_item.sprite);

            vertices.extend_from_slice(&quad);
            indices.extend_from_slice(&[
                base_vertex,
                base_vertex + 1,
                base_vertex + 2,
                base_vertex + 2,
                base_vertex + 3,
                base_vertex,
            ]);

            let should_merge = match batches.last() {
                Some(last) => Arc::ptr_eq(&last.bind_group, &sprite_item.texture_bind_group),
                None => false,
            };

            if should_merge {
                if let Some(last) = batches.last_mut() {
                    last.index_count += 6;
                }
            } else {
                let index_start = (indices.len() - 6) as u32;
                batches.push(DrawBatch {
                    bind_group: Arc::clone(&sprite_item.texture_bind_group),
                    index_start,
                    index_count: 6,
                });
            }
        }

        (vertices, indices, batches)
    }
}
