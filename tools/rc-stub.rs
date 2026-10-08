use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut out_file = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        let lower = arg.to_lowercase();
        if lower == "/fo" || lower == "-fo" {
            if i + 1 < args.len() {
                out_file = Some(args[i + 1].clone());
                i += 1;
            }
        } else if lower.starts_with("/fo") || lower.starts_with("-fo") {
            out_file = Some(arg[3..].to_string());
        }
        i += 1;
    }

    if let Some(path_str) = out_file {
        let path = Path::new(&path_str);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if path_str.ends_with(".res") {
            // Standard null resource header for Windows .res files (32 bytes)
            let res_header: [u8; 32] = [
                0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00,
                0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];
            let _ = fs::write(path, &res_header);
        } else {
            // Standard COFF archive header for .lib files (8 bytes: !<arch>\n)
            let _ = fs::write(path, b"!<arch>\n");
        }
    }
}
