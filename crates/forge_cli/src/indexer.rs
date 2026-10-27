//! Indexing engine for Forge game projects.
//!
//! Generates `.forge/map.json` indexing scenes, entities, components, scripts, assets,
//! animation clips, and input actions. Deterministic and fast.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Entity representation in the project map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapEntity {
    pub name: String,
    pub components: Vec<String>,
}

/// Script representation in the project map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapScript {
    pub path: String,
    pub exports: Vec<String>,
    pub lifecycle_functions: Vec<String>,
}

/// Animation clip representation in the project map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapAnimationClip {
    pub name: String,
    pub path: String,
    pub fps: u32,
    pub frame_count: usize,
    pub loop_mode: String,
}

/// Deterministic map index of a Forge project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProjectMap {
    pub scenes: Vec<String>,
    pub entities: BTreeMap<String, MapEntity>,
    pub scripts: Vec<MapScript>,
    pub assets: Vec<String>,
    pub animation_clips: Vec<MapAnimationClip>,
    pub input_actions: Vec<String>,
}

/// Builds the deterministic ProjectMap for a given project directory.
pub fn build_project_map(project_dir: &Path) -> Result<ProjectMap, Box<dyn std::error::Error>> {
    let mut map = ProjectMap::default();

    // 1. Scan scenes (.json files representing scenes)
    let mut scene_paths = Vec::new();
    find_files_by_name_or_ext(project_dir, &["scene.json"], &mut scene_paths)?;
    let mut rel_scenes = Vec::new();
    for p in scene_paths {
        if let Ok(rel) = p.strip_prefix(project_dir) {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            rel_scenes.push((rel_str, p));
        }
    }
    rel_scenes.sort_by(|a, b| a.0.cmp(&b.0));

    for (rel_str, full_path) in rel_scenes {
        map.scenes.push(rel_str);
        if let Ok(content) = fs::read_to_string(&full_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(entities_arr) = val.get("entities").and_then(|e| e.as_array()) {
                    for entity in entities_arr {
                        let id = entity.get("id").and_then(|i| i.as_str()).unwrap_or("");
                        let name_from_field = entity.get("name").and_then(|n| n.as_str());
                        let comps_obj = entity.get("components").and_then(|c| c.as_object());

                        let mut comp_names = BTreeSet::new();
                        let mut name_from_comp = None;
                        if let Some(comps) = comps_obj {
                            for (c_name, c_val) in comps {
                                comp_names.insert(c_name.clone());
                                if c_name == "Name" {
                                    if let Some(s) = c_val.as_str() {
                                        name_from_comp = Some(s.to_string());
                                    } else if let Some(s) = c_val.get("0").and_then(|v| v.as_str()) {
                                        name_from_comp = Some(s.to_string());
                                    }
                                }
                            }
                        }

                        let final_name = name_from_field
                            .map(|s| s.to_string())
                            .or(name_from_comp)
                            .unwrap_or_else(|| {
                                if !id.is_empty() {
                                    id.to_string()
                                } else {
                                    "Entity".to_string()
                                }
                            });

                        let entity_key = if !id.is_empty() {
                            id.to_string()
                        } else {
                            final_name.clone()
                        };

                        map.entities.insert(
                            entity_key,
                            MapEntity {
                                name: final_name,
                                components: comp_names.into_iter().collect(),
                            },
                        );
                    }
                }
            }
        }
    }

    // 2. Scan scripts (.ts files in project, excluding forge.d.ts)
    let mut ts_files = Vec::new();
    find_files_by_ext(project_dir, &["ts", "js"], &mut ts_files)?;
    let mut rel_scripts = Vec::new();
    for p in ts_files {
        let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name.ends_with(".d.ts") {
            continue;
        }
        if let Ok(rel) = p.strip_prefix(project_dir) {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if rel_str.starts_with(".forge/") || rel_str.starts_with("node_modules/") {
                continue;
            }
            rel_scripts.push((rel_str, p));
        }
    }
    rel_scripts.sort_by(|a, b| a.0.cmp(&b.0));

    for (rel_str, full_path) in rel_scripts {
        let mut exports = BTreeSet::new();
        let mut lifecycle_fns = BTreeSet::new();

        if let Ok(source) = fs::read_to_string(&full_path) {
            extract_script_symbols(&source, &mut exports, &mut lifecycle_fns);
        }

        map.scripts.push(MapScript {
            path: rel_str,
            exports: exports.into_iter().collect(),
            lifecycle_functions: lifecycle_fns.into_iter().collect(),
        });
    }

    // 3. Scan assets
    let mut asset_paths = Vec::new();
    find_files_by_ext(
        project_dir,
        &[
            "png", "jpg", "jpeg", "webp", "wav", "mp3", "ogg", "flac", "ttf", "otf", "json",
        ],
        &mut asset_paths,
    )?;
    let mut rel_assets = Vec::new();
    for p in asset_paths {
        if let Ok(rel) = p.strip_prefix(project_dir) {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if rel_str.starts_with(".forge/")
                || rel_str.starts_with("node_modules/")
                || rel_str == "input_map.json"
                || rel_str == "package.json"
                || rel_str == "tsconfig.json"
                || rel_str.ends_with("scene.json")
            {
                continue;
            }
            rel_assets.push((rel_str, p));
        }
    }
    rel_assets.sort_by(|a, b| a.0.cmp(&b.0));

    for (rel_str, full_path) in rel_assets {
        if rel_str.ends_with(".json") {
            if let Ok(content) = fs::read_to_string(&full_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if val.get("frames").and_then(|f| f.as_array()).is_some()
                        && val.get("fps").is_some()
                    {
                        let name = val
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or_else(|| {
                                full_path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("clip")
                            });
                        let fps = val.get("fps").and_then(|f| f.as_f64()).unwrap_or(12.0) as u32;
                        let frames_len = val
                            .get("frames")
                            .and_then(|f| f.as_array())
                            .map(|a| a.len())
                            .unwrap_or(0);
                        let loop_mode = val
                            .get("loop_mode")
                            .and_then(|l| l.as_str())
                            .unwrap_or("loop")
                            .to_string();

                        map.animation_clips.push(MapAnimationClip {
                            name: name.to_string(),
                            path: rel_str.clone(),
                            fps,
                            frame_count: frames_len,
                            loop_mode,
                        });
                    }
                }
            }
        }
        map.assets.push(rel_str);
    }
    map.animation_clips.sort_by(|a, b| a.name.cmp(&b.name));

    // 4. Scan input actions (input_map.json)
    let input_map_path = project_dir.join("input_map.json");
    if input_map_path.exists() {
        if let Ok(content) = fs::read_to_string(&input_map_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(actions_obj) = val.get("actions").and_then(|a| a.as_object()) {
                    let mut action_names: Vec<String> = actions_obj.keys().cloned().collect();
                    action_names.sort();
                    map.input_actions = action_names;
                }
            }
        }
    }

    Ok(map)
}

/// Recursively generates and writes `.forge/map.json`.
pub fn index_project(project_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let map = build_project_map(project_dir)?;
    let forge_dir = project_dir.join(".forge");
    fs::create_dir_all(&forge_dir)?;
    let map_file = forge_dir.join("map.json");
    let json_bytes = serde_json::to_vec_pretty(&map)?;
    fs::write(&map_file, json_bytes)?;
    Ok(map_file)
}

fn extract_script_symbols(
    source: &str,
    exports: &mut BTreeSet<String>,
    lifecycle_fns: &mut BTreeSet<String>,
) {
    let known_lifecycles = [
        "onStart",
        "onUpdate",
        "onFixedUpdate",
        "onCollisionEnter",
        "onCollisionExit",
        "onTriggerEnter",
        "onDestroy",
        "serialize",
        "deserialize",
    ];

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("export ") {
            continue;
        }
        let after_export = trimmed.strip_prefix("export ").unwrap().trim();

        let fn_decl = if after_export.starts_with("function ") {
            Some(after_export.strip_prefix("function ").unwrap().trim())
        } else if after_export.starts_with("async function ") {
            Some(after_export.strip_prefix("async function ").unwrap().trim())
        } else {
            None
        };

        if let Some(rest) = fn_decl {
            let name_end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            let name = &rest[..name_end];
            if !name.is_empty() {
                exports.insert(name.to_string());
                if known_lifecycles.contains(&name) {
                    lifecycle_fns.insert(name.to_string());
                }
            }
            continue;
        }

        for kw in ["const ", "let ", "var "] {
            if after_export.starts_with(kw) {
                let rest = after_export.strip_prefix(kw).unwrap().trim();
                let name_end = rest
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(rest.len());
                let name = &rest[..name_end];
                if !name.is_empty() {
                    exports.insert(name.to_string());
                }
                break;
            }
        }
    }
}

fn find_files_by_name_or_ext(
    dir: &Path,
    target_names: &[&str],
    results: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == ".forge" || name_str == "node_modules" || name_str == ".git" {
                continue;
            }
            find_files_by_name_or_ext(&path, target_names, results)?;
        } else if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if target_names.contains(&name) {
                    results.push(path);
                }
            }
        }
    }
    Ok(())
}

fn find_files_by_ext(
    dir: &Path,
    extensions: &[&str],
    results: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == ".forge" || name_str == "node_modules" || name_str == ".git" {
                continue;
            }
            find_files_by_ext(&path, extensions, results)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    results.push(path);
                }
            }
        }
    }
    Ok(())
}
