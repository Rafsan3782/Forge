//! Texture asset management, caching by asset ID, PNG decoding, and fallback handling.

use crate::error::AssetError;
use forge_render::{GpuTexture, RenderContext};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Central storage and caching system for 2D textures.
pub struct TextureStore {
    textures: HashMap<String, GpuTexture>,
    magenta_placeholder: GpuTexture,
    white_pixel: GpuTexture,
    base_dir: Option<PathBuf>,
    database: Option<crate::database::AssetDatabase>,
}

impl TextureStore {
    /// Initializes a new texture store with a visible magenta placeholder texture.
    pub fn new(render_context: &RenderContext) -> Result<Self, AssetError> {
        let magenta_placeholder = GpuTexture::create_magenta_placeholder(
            &render_context.device,
            &render_context.queue,
            &render_context.texture_bind_group_layout,
            &render_context.default_sampler,
        )?;

        let white_pixel = GpuTexture::create_white_pixel(
            &render_context.device,
            &render_context.queue,
            &render_context.texture_bind_group_layout,
            &render_context.default_sampler,
        )?;

        Ok(Self {
            textures: HashMap::new(),
            magenta_placeholder,
            white_pixel,
            base_dir: None,
            database: None,
        })
    }

    /// Associates an asset database with this texture store to resolve UUIDs.
    pub fn set_database(&mut self, database: crate::database::AssetDatabase) {
        self.database = Some(database);
    }

    /// Sets the base directory for resolving relative asset paths.
    pub fn set_base_dir(&mut self, dir: impl Into<PathBuf>) {
        self.base_dir = Some(dir.into());
    }

    /// Gets the current base directory if set.
    pub fn base_dir(&self) -> Option<&Path> {
        self.base_dir.as_deref()
    }

    /// Returns the magenta placeholder texture used when assets are missing.
    pub fn magenta_placeholder(&self) -> &GpuTexture {
        &self.magenta_placeholder
    }

    /// Returns the white 1x1 texture used for drawing solid colored geometry and debug lines.
    pub fn white_pixel(&self) -> &GpuTexture {
        &self.white_pixel
    }

    /// Explicitly registers a loaded GPU texture under an asset ID.
    pub fn insert(&mut self, asset_id: impl Into<String>, texture: GpuTexture) {
        self.textures.insert(asset_id.into(), texture);
    }

    /// Decodes a PNG image from in-memory bytes and uploads it to the GPU.
    pub fn load_png_from_memory(
        &mut self,
        asset_id: &str,
        bytes: &[u8],
        render_context: &RenderContext,
    ) -> Result<GpuTexture, AssetError> {
        let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let texture = GpuTexture::from_rgba8(
            &render_context.device,
            &render_context.queue,
            &render_context.texture_bind_group_layout,
            &render_context.default_sampler,
            &rgba,
            width,
            height,
            Some(asset_id),
        )?;

        self.textures.insert(asset_id.to_string(), texture.clone());
        Ok(texture)
    }

    /// Loads a PNG texture from disk and uploads it to the GPU.
    pub fn load_png_from_file(
        &mut self,
        asset_id: &str,
        path: impl AsRef<Path>,
        render_context: &RenderContext,
    ) -> Result<GpuTexture, AssetError> {
        let bytes = std::fs::read(path)?;
        self.load_png_from_memory(asset_id, &bytes, render_context)
    }

    /// Retrieves a cached texture or attempts to load it from disk.
    ///
    /// If the asset cannot be found or decoded, a warning is logged with `tracing`
    /// and the magenta placeholder is returned and cached for subsequent requests.
    pub fn get_or_load(&mut self, asset_id: &str, render_context: &RenderContext) -> GpuTexture {
        if let Some(texture) = self.textures.get(asset_id) {
            return texture.clone();
        }

        // Search for file on disk across base dir, database resolution, direct path, and parent directories
        let mut candidate_paths = Vec::new();

        if let Some(db) = &self.database {
            if let Some(resolved) = db.resolve_path(asset_id) {
                candidate_paths.push(resolved);
            }
        }

        if let Some(base) = &self.base_dir {
            candidate_paths.push(base.join(asset_id));
        }

        let direct_path = PathBuf::from(asset_id);
        candidate_paths.push(direct_path.clone());

        // Check relative to current working directory parents (workspace root discovery)
        if let Ok(cwd) = std::env::current_dir() {
            let mut curr = Some(cwd);
            while let Some(dir) = curr {
                let candidate = dir.join(asset_id);
                if !candidate_paths.contains(&candidate) {
                    candidate_paths.push(candidate);
                }
                curr = dir.parent().map(|p| p.to_path_buf());
            }
        }

        let mut resolved_path = None;
        for path in &candidate_paths {
            if path.exists() && path.is_file() {
                resolved_path = Some(path.clone());
                break;
            }
        }

        match resolved_path {
            Some(path) => match self.load_png_from_file(asset_id, &path, render_context) {
                Ok(tex) => tex,
                Err(err) => {
                    tracing::warn!(
                        "Failed to decode or upload PNG asset '{}' from {:?}: {}. Using magenta placeholder.",
                        asset_id,
                        path,
                        err
                    );
                    self.textures
                        .insert(asset_id.to_string(), self.magenta_placeholder.clone());
                    self.magenta_placeholder.clone()
                }
            },
            None => {
                tracing::warn!(
                    "Asset '{}' not found in candidate paths. Using magenta placeholder.",
                    asset_id
                );
                self.textures
                    .insert(asset_id.to_string(), self.magenta_placeholder.clone());
                self.magenta_placeholder.clone()
            }
        }
    }

    /// Reloads a texture from disk, updating the GPU texture and cache.
    pub fn reload(
        &mut self,
        asset_id: &str,
        render_context: &RenderContext,
    ) -> Result<GpuTexture, AssetError> {
        self.textures.remove(asset_id);
        let tex = self.get_or_load(asset_id, render_context);
        Ok(tex)
    }

    /// Removes a texture from cache.
    pub fn remove(&mut self, asset_id: &str) {
        self.textures.remove(asset_id);
    }
}
