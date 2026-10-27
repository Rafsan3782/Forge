use forge_cli::{build_project_map, index_project};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[test]
fn test_forge_index_deterministic_output() {
    let template_path = Path::new("templates/platformer");
    if !template_path.exists() {
        return;
    }

    let map1 = build_project_map(template_path).expect("build map 1");
    let json1 = serde_json::to_string_pretty(&map1).expect("serialize map 1");

    let map2 = build_project_map(template_path).expect("build map 2");
    let json2 = serde_json::to_string_pretty(&map2).expect("serialize map 2");

    // Must be completely deterministic
    assert_eq!(json1, json2, "Project map generation must be deterministic");

    // Must have platformer content
    assert!(map1.scenes.contains(&"scene.json".to_string()));
    assert!(!map1.entities.is_empty(), "Entities should not be empty");
    assert!(!map1.scripts.is_empty(), "Scripts should not be empty");
    assert!(!map1.assets.is_empty(), "Assets should not be empty");
    assert!(map1.input_actions.contains(&"jump".to_string()));
}

#[test]
fn test_forge_index_performance_under_budget() {
    let temp_dir = std::env::temp_dir().join("forge_test_perf_index");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let mut entities = Vec::new();
    for i in 0..1000 {
        entities.push(serde_json::json!({
            "id": format!("entity_{:04}", i),
            "name": format!("Entity {:04}", i),
            "components": {
                "Transform2D": {
                    "position": [i as f64, 0.0],
                    "rotation": 0.0,
                    "scale": [1.0, 1.0]
                },
                "Sprite": {
                    "asset_id": "test.png"
                }
            }
        }));
    }
    let scene_json = serde_json::json!({
        "name": "BigScene",
        "entities": entities
    });
    fs::write(
        temp_dir.join("scene.json"),
        serde_json::to_string(&scene_json).unwrap(),
    )
    .unwrap();

    let start = Instant::now();
    let map_file = index_project(&temp_dir).expect("index project");
    let elapsed = start.elapsed();

    assert!(map_file.exists());
    assert!(
        elapsed.as_millis() < 200,
        "Indexing 1000 entities took {:?} which exceeds 200ms budget",
        elapsed
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
