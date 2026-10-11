//! Asset metadata sidecar (`name.ext.meta.json`) and asset database indexing.
//!
//! Every asset file under a project's `assets/` directory receives a stable UUID
//! stored in a sidecar `.meta.json` file. Moving or renaming the asset alongside
//! its `.meta.json` preserves the asset ID.

use crate::error::AssetError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Extension suffix for asset metadata files.
pub const META_EXTENSION: &str = ".meta.json";

/// Type of asset tracked in the asset database.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    /// Texture image (e.g. PNG).
    Texture,
    /// Sprite sheet or grid/atlas definition.
    SpriteSheet,
    /// Animation clip defining keyframes and frame rate.
    Animation,
    /// Scene JSON hierarchy.
    Scene,
    /// Audio sound or music file.
    Audio,
    /// Other / generic asset.
    Other(String),
}

impl AssetType {
    /// Infers the default asset type from a file path extension or content.
    pub fn infer_from_path(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "png" | "jpg" | "jpeg" => AssetType::Texture,
            "json" => {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if stem.ends_with(".spritesheet") || stem.ends_with(".atlas") {
                    AssetType::SpriteSheet
                } else if stem.ends_with(".anim") || stem.ends_with(".animation") {
                    AssetType::Animation
                } else if stem.ends_with(".scene") || stem == "scene" {
                    AssetType::Scene
                } else {
                    AssetType::Other("json".to_string())
                }
            }
            "ogg" | "wav" | "mp3" => AssetType::Audio,
            other => AssetType::Other(other.to_string()),
        }
    }

    /// Returns string representation.
    pub fn as_str(&self) -> &str {
        match self {
            AssetType::Texture => "texture",
            AssetType::SpriteSheet => "spritesheet",
            AssetType::Animation => "animation",
            AssetType::Scene => "scene",
            AssetType::Audio => "audio",
            AssetType::Other(s) => s.as_str(),
        }
    }
}

/// Persistent metadata sidecar (`name.ext.meta.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetMeta {
    /// Stable UUID identifying the asset across renames and moves.
    pub id: String,
    /// Classified asset type.
    pub asset_type: AssetType,
    /// Optional import settings and format-specific parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_settings: Option<serde_json::Value>,
}

impl AssetMeta {
    /// Creates a new AssetMeta with a generated UUID v4.
    pub fn new(asset_type: AssetType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            asset_type,
            import_settings: None,
        }
    }

    /// Creates an AssetMeta with a specific ID.
    pub fn with_id(id: impl Into<String>, asset_type: AssetType) -> Self {
        Self {
            id: id.into(),
            asset_type,
            import_settings: None,
        }
    }

    /// Loads metadata from the sidecar file path.
    pub fn load_from_file(meta_path: impl AsRef<Path>) -> Result<Self, AssetError> {
        let content = fs::read_to_string(meta_path)?;
        let meta = serde_json::from_str(&content)?;
        Ok(meta)
    }

    /// Saves metadata to the sidecar file path.
    pub fn save_to_file(&self, meta_path: impl AsRef<Path>) -> Result<(), AssetError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(meta_path, content)?;
        Ok(())
    }
}

/// An entry indexed in the AssetDatabase.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetEntry {
    /// Stable asset UUID.
    pub id: String,
    /// Absolute or base-dir-relative path to the actual asset file.
    pub file_path: PathBuf,
    /// Path to the sidecar `.meta.json` file.
    pub meta_path: PathBuf,
    /// Parsed sidecar metadata.
    pub meta: AssetMeta,
}

use forge_core::Resource;

/// Asset Database indexing all assets and `.meta.json` files within a root folder.
#[derive(Resource, Debug, Default, Clone)]
pub struct AssetDatabase {
    /// Root directory being indexed (e.g. `templates/hello/assets` or `assets/`).
    root_dir: PathBuf,
    /// Mapping from Asset ID (UUID) -> AssetEntry.
    id_to_entry: HashMap<String, AssetEntry>,
    /// Mapping from file path (canonicalized or normalized) -> Asset ID.
    path_to_id: HashMap<PathBuf, String>,
}

impl AssetDatabase {
    /// Creates a new empty AssetDatabase for the specified root directory.
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
            id_to_entry: HashMap::new(),
            path_to_id: HashMap::new(),
        }
    }

    /// Scans the root directory recursively, generating `.meta.json` files for any assets
    /// missing one, and indexing all assets into memory.
    pub fn scan_and_index(&mut self) -> Result<(), AssetError> {
        self.id_to_entry.clear();
        self.path_to_id.clear();

        if !self.root_dir.exists() {
            return Ok(());
        }

        self.index_dir_recursive(&self.root_dir.clone())
    }

    fn index_dir_recursive(&mut self, dir: &Path) -> Result<(), AssetError> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.index_dir_recursive(&path)?;
            } else {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Skip sidecar meta files and hidden files
                if filename.ends_with(META_EXTENSION) || filename.starts_with('.') {
                    continue;
                }

                let meta_path = PathBuf::from(format!("{}{}", path.display(), META_EXTENSION));
                let meta = if meta_path.exists() {
                    match AssetMeta::load_from_file(&meta_path) {
                        Ok(m) => m,
                        Err(e) => {
                            tracing::warn!(
                                "Corrupt meta file {:?}: {}. Re-generating with new UUID.",
                                meta_path,
                                e
                            );
                            let m = AssetMeta::new(AssetType::infer_from_path(&path));
                            m.save_to_file(&meta_path)?;
                            m
                        }
                    }
                } else {
                    let m = AssetMeta::new(AssetType::infer_from_path(&path));
                    m.save_to_file(&meta_path)?;
                    m
                };

                let entry = AssetEntry {
                    id: meta.id.clone(),
                    file_path: path.clone(),
                    meta_path,
                    meta,
                };

                self.path_to_id.insert(path.clone(), entry.id.clone());
                self.id_to_entry.insert(entry.id.clone(), entry);
            }
        }

        Ok(())
    }

    /// Resolves an asset file path by asset ID (UUID) or fallback file path.
    pub fn resolve_path(&self, id_or_path: &str) -> Option<PathBuf> {
        if let Some(entry) = self.id_to_entry.get(id_or_path) {
            return Some(entry.file_path.clone());
        }

        // Try direct file path
        let direct = PathBuf::from(id_or_path);
        if direct.exists() {
            return Some(direct);
        }

        let relative = self.root_dir.join(id_or_path);
        if relative.exists() {
            return Some(relative);
        }

        None
    }

    /// Looks up asset entry by ID.
    pub fn get(&self, id: &str) -> Option<&AssetEntry> {
        self.id_to_entry.get(id)
    }

    /// Finds asset ID from a file path.
    pub fn get_id_by_path(&self, path: &Path) -> Option<&str> {
        self.path_to_id.get(path).map(|s| s.as_str())
    }

    /// Returns all registered asset entries.
    pub fn all_entries(&self) -> impl Iterator<Item = &AssetEntry> {
        self.id_to_entry.values()
    }

    /// Imports a file into the target directory, copying it and writing its sidecar meta.
    pub fn import_file(
        &mut self,
        source_path: &Path,
        target_path: &Path,
        asset_type: Option<AssetType>,
        metadata: Option<serde_json::Value>,
    ) -> Result<AssetEntry, AssetError> {
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(source_path, target_path)?;

        let meta_path = PathBuf::from(format!("{}{}", target_path.display(), META_EXTENSION));
        let inferred_type = asset_type.unwrap_or_else(|| AssetType::infer_from_path(target_path));
        let mut meta = AssetMeta::new(inferred_type);
        meta.import_settings = metadata;
        meta.save_to_file(&meta_path)?;

        let entry = AssetEntry {
            id: meta.id.clone(),
            file_path: target_path.to_path_buf(),
            meta_path,
            meta,
        };

        self.path_to_id
            .insert(target_path.to_path_buf(), entry.id.clone());
        self.id_to_entry.insert(entry.id.clone(), entry.clone());

        Ok(entry)
    }
}
