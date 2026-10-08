fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let res_lib = std::path::Path::new(&out_dir).join("resource.lib");
        if !res_lib.exists() {
            let _ = std::fs::write(&res_lib, b"!<arch>\n");
        }
    }
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!(r#"cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"#);
    println!("cargo:rustc-link-arg=/ignore:4099");
    tauri_build::build();
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let res_lib = std::path::Path::new(&out_dir).join("resource.lib");
        if !res_lib.exists() {
            let _ = std::fs::write(&res_lib, b"!<arch>\n");
        }
    }
}
