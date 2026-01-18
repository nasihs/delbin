//! Basic usage example

use delbin::{generate, hex_dump, Value};
use std::collections::HashMap;

fn main() {
    let dsl = r#"
        @endian = little;

        struct header @packed {
            magic:          [u8; 4] = @bytes("fpk\0");
            image_type:     u32 = 0;
            header_ver:     u16 = 0x0100;
            header_size:    u16 = @sizeof(@self);
            fw_version:     u32 = (${VERSION_MAJOR} << 24) | (${VERSION_MINOR} << 16) | ${VERSION_PATCH};
            build_number:   u32 = ${BUILD_NUMBER};
            version_str:    [u8; 16] = @bytes(${VERSION_STRING});
            flags:          u32 = 0;
            img_size:       u32 = @sizeof(image);
            packed_size:    u32 = @sizeof(image);
            timestamp:      u32 = ${UNIX_STAMP};
            partition:      [u8; 16] = @bytes("app");
            watermark:      [u8; 16] = @bytes("DELBIN_DEMO");
            reserved:       [u8; 32];
            img_crc32:      u32 = @crc32(image);
            img_sha256:     [u8; 32] = @sha256(image);
            header_crc32:   u32 = @crc32(@self[..header_crc32]);
            _padding:       [u8; 256 - @offsetof(_padding)];
        }
    "#;

    // Set environment variables
    let mut env = HashMap::new();
    env.insert("VERSION_MAJOR".to_string(), Value::U64(1));
    env.insert("VERSION_MINOR".to_string(), Value::U64(2));
    env.insert("VERSION_PATCH".to_string(), Value::U64(3));
    env.insert("BUILD_NUMBER".to_string(), Value::U64(100));
    env.insert(
        "VERSION_STRING".to_string(),
        Value::String("1.2.3-build100".to_string()),
    );
    env.insert("UNIX_STAMP".to_string(), Value::U64(1705574400));

    // Set sections
    let mut sections = HashMap::new();
    sections.insert("image".to_string(), vec![0xABu8; 1024]);

    // Generate
    match generate(dsl, &env, &sections) {
        Ok(result) => {
            println!("Generated header ({} bytes):", result.data.len());
            println!("{}", hex_dump(&result.data, 16));

            if !result.warnings.is_empty() {
                println!("\nWarnings:");
                for w in &result.warnings {
                    println!("  [{:?}] {}", w.code, w.message);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            if let Some(hint) = &e.hint {
                eprintln!("Hint: {}", hint);
            }
        }
    }
}
