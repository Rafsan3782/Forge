use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut out_file = None;
    let mut rc_file = None;

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
        } else if !arg.starts_with('/') && !arg.starts_with('-') && arg.ends_with(".rc") {
            rc_file = Some(arg.clone());
        }
        i += 1;
    }

    if let Some(path_str) = out_file {
        let path = Path::new(&path_str);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Try to locate icon file from rc_file or common locations
        let mut icon_path: Option<PathBuf> = None;
        if let Some(ref rc) = rc_file {
            if let Ok(content) = fs::read_to_string(rc) {
                for line in content.lines() {
                    if line.contains("ICON") {
                        if let Some(start) = line.find('"') {
                            if let Some(end) = line[start + 1..].find('"') {
                                let p = &line[start + 1..start + 1 + end];
                                let cand = PathBuf::from(p);
                                if cand.exists() {
                                    icon_path = Some(cand);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut res_data = Vec::new();
        // 1. Standard null resource header (32 bytes)
        let res_header: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00,
            0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        res_data.extend_from_slice(&res_header);

        // 2. If icon found, encode RT_GROUP_ICON and RT_ICON resources
        if let Some(ref ip) = icon_path {
            if let Ok(ico_bytes) = fs::read(ip) {
                if ico_bytes.len() >= 6 && ico_bytes[2] == 1 && ico_bytes[3] == 0 {
                    let count = u16::from_le_bytes([ico_bytes[4], ico_bytes[5]]) as usize;
                    if ico_bytes.len() >= 6 + count * 16 {
                        // Build GRPICONDIR + GRPICONDIRENTRY
                        let mut grp_data = Vec::new();
                        grp_data.extend_from_slice(&ico_bytes[0..6]); // Reserved + Type + Count
                        for idx in 0..count {
                            let entry_offset = 6 + idx * 16;
                            grp_data.extend_from_slice(&ico_bytes[entry_offset..entry_offset + 12]); // Width..BytesInRes
                            let n_id = (idx + 1) as u16;
                            grp_data.extend_from_slice(&n_id.to_le_bytes()); // nID
                        }

                        // Write RT_GROUP_ICON (Type 14, ID 32512)
                        write_res_entry(&mut res_data, 14, 32512, &grp_data);

                        // Write each RT_ICON (Type 3, ID 1..count)
                        for idx in 0..count {
                            let entry_offset = 6 + idx * 16;
                            let bytes_in_res = u32::from_le_bytes([
                                ico_bytes[entry_offset + 8],
                                ico_bytes[entry_offset + 9],
                                ico_bytes[entry_offset + 10],
                                ico_bytes[entry_offset + 11],
                            ]) as usize;
                            let image_offset = u32::from_le_bytes([
                                ico_bytes[entry_offset + 12],
                                ico_bytes[entry_offset + 13],
                                ico_bytes[entry_offset + 14],
                                ico_bytes[entry_offset + 15],
                            ]) as usize;

                            if image_offset + bytes_in_res <= ico_bytes.len() {
                                let img_data = &ico_bytes[image_offset..image_offset + bytes_in_res];
                                write_res_entry(&mut res_data, 3, (idx + 1) as u16, img_data);
                            }
                        }
                    }
                }
            }
        }

        let _ = fs::write(path, &res_data);
    }
}

fn write_res_entry(out: &mut Vec<u8>, type_id: u16, name_id: u16, data: &[u8]) {
    let header_size: u32 = 32;
    let data_size: u32 = data.len() as u32;

    out.extend_from_slice(&data_size.to_le_bytes());
    out.extend_from_slice(&header_size.to_le_bytes());
    // Type: 0xFFFF followed by type_id
    out.extend_from_slice(&[0xff, 0xff]);
    out.extend_from_slice(&type_id.to_le_bytes());
    // Name: 0xFFFF followed by name_id
    out.extend_from_slice(&[0xff, 0xff]);
    out.extend_from_slice(&name_id.to_le_bytes());
    // DataVersion (u32)
    out.extend_from_slice(&0u32.to_le_bytes());
    // MemoryFlags (u16): 0x1030 (MOVEABLE | PURE | DISCARDABLE)
    out.extend_from_slice(&0x1030u16.to_le_bytes());
    // LanguageId (u16): 0 (Neutral)
    out.extend_from_slice(&0u16.to_le_bytes());
    // Version (u32)
    out.extend_from_slice(&0u32.to_le_bytes());
    // Characteristics (u32)
    out.extend_from_slice(&0u32.to_le_bytes());

    // Data
    out.extend_from_slice(data);
    // DWORD align data
    let rem = data.len() % 4;
    if rem != 0 {
        let padding = 4 - rem;
        out.extend(std::iter::repeat(0u8).take(padding));
    }
}
