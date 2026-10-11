//! Tests for AssetDatabase ID stability, metadata persistence, and file renaming.

use forge_assets::{AssetDatabase, AssetType, META_EXTENSION};
use std::fs;

#[test]
fn test_asset_database_id_stability_on_rename() {
    let temp_dir = std::env::temp_dir().join(format!("forge_test_db_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let file_path = temp_dir.join("player.png");
    fs::write(&file_path, b"dummy png content").unwrap();

    let mut db = AssetDatabase::new(&temp_dir);
    db.scan_and_index().unwrap();

    let initial_id = db
        .get_id_by_path(&file_path)
        .expect("Expected player.png to be indexed")
        .to_string();
    assert!(!initial_id.is_empty());

    // Verify sidecar .meta.json exists
    let meta_path = temp_dir.join(format!("player.png{}", META_EXTENSION));
    assert!(meta_path.exists());

    // Rename file and its .meta.json sidecar together
    let renamed_file = temp_dir.join("player_renamed.png");
    let renamed_meta = temp_dir.join(format!("player_renamed.png{}", META_EXTENSION));
    fs::rename(&file_path, &renamed_file).unwrap();
    fs::rename(&meta_path, &renamed_meta).unwrap();

    // Rescan database
    let mut db2 = AssetDatabase::new(&temp_dir);
    db2.scan_and_index().unwrap();

    // Asset ID must remain identical
    let preserved_id = db2
        .get_id_by_path(&renamed_file)
        .expect("Expected renamed file to be indexed");
    assert_eq!(preserved_id, initial_id);

    // Resolving by original UUID should point to the new path
    let resolved_path = db2
        .resolve_path(&initial_id)
        .expect("Should resolve by UUID");
    assert_eq!(resolved_path, renamed_file);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_asset_import() {
    let temp_dir = std::env::temp_dir().join(format!("forge_test_import_{}", uuid::Uuid::new_v4()));
    let src_dir = temp_dir.join("src");
    let target_dir = temp_dir.join("assets");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&target_dir).unwrap();

    let src_file = src_dir.join("sample.png");
    fs::write(&src_file, b"sample png").unwrap();

    let target_file = target_dir.join("textures/sample.png");

    let mut db = AssetDatabase::new(&target_dir);
    let entry = db
        .import_file(&src_file, &target_file, Some(AssetType::Texture), None)
        .unwrap();

    assert!(target_file.exists());
    assert_eq!(entry.meta.asset_type, AssetType::Texture);
    assert_eq!(db.resolve_path(&entry.id), Some(target_file));

    let _ = fs::remove_dir_all(&temp_dir);
}
