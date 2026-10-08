fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let res_lib = std::path::Path::new(&out_dir).join("resource.lib");
        if !res_lib.exists() {
            let _ = std::fs::write(&res_lib, b"!<arch>\n");
        }
    }
    tauri_build::build();
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let res_lib = std::path::Path::new(&out_dir).join("resource.lib");
        if !res_lib.exists() {
            let _ = std::fs::write(&res_lib, b"!<arch>\n");
        }
    }
}
