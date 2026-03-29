//! CRC algorithm examples
//!
//! Demonstrates the unified `@crc("algorithm", range)` function alongside
//! the `@crc32()` shorthand, using both section data and self-referencing ranges.

use delbin::{generate, hex_dump};
use std::collections::HashMap;

fn main() {
    let image = vec![0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let mut sections = HashMap::new();
    sections.insert("image".to_string(), image.clone());

    // ── Example 1: @crc("crc32", ...) is identical to @crc32() ──────────

    let dsl_unified = r#"
        @endian = little;
        struct header @packed {
            magic:    [u8; 4] = @bytes("TEST");
            crc32_a:  u32 = @crc32(image);
            crc32_b:  u32 = @crc("crc32", image);
        }
    "#;

    match generate(dsl_unified, &HashMap::new(), &sections) {
        Ok(result) => {
            println!("=== Example 1: @crc32() vs @crc(\"crc32\", ...) ===");
            println!("{}", hex_dump(&result.data, 16));
            let crc_a = u32::from_le_bytes(result.data[4..8].try_into().unwrap());
            let crc_b = u32::from_le_bytes(result.data[8..12].try_into().unwrap());
            println!("crc32_a = 0x{crc_a:08X}");
            println!("crc32_b = 0x{crc_b:08X}");
            assert_eq!(crc_a, crc_b, "@crc32() and @crc(\"crc32\",...) must match");
            println!("✓ Both produce identical output\n");
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    // ── Example 2: CRC16-MODBUS over an external section ────────────────

    let dsl_crc16 = r#"
        @endian = little;
        struct header @packed {
            magic:  [u8; 4] = @bytes("HDR\0");
            length: u32 = @sizeof(image);
            crc16:  u16 = @crc("crc16-modbus", image);
        }
    "#;

    match generate(dsl_crc16, &HashMap::new(), &sections) {
        Ok(result) => {
            println!("=== Example 2: @crc(\"crc16-modbus\", image) ===");
            println!("{}", hex_dump(&result.data, 16));
            let crc16 = u16::from_le_bytes(result.data[8..10].try_into().unwrap());
            println!("crc16-modbus = 0x{crc16:04X}\n");
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    // ── Example 3: Self-referencing partial range ────────────────────────
    //  Compute CRC from 'magic' up to (not including) 'body_crc'

    let dsl_partial = r#"
        @endian = little;
        struct header @packed {
            magic:    [u8; 4] = @bytes("TEST");
            reserved: u32     = 0xDEADBEEF;
            body_crc: u32     = @crc32(@self[magic..body_crc]);
        }
    "#;

    match generate(dsl_partial, &HashMap::new(), &HashMap::new()) {
        Ok(result) => {
            println!("=== Example 3: @crc32(@self[magic..body_crc]) ===");
            println!("{}", hex_dump(&result.data, 16));
            let crc = u32::from_le_bytes(result.data[8..12].try_into().unwrap());
            println!("body_crc = 0x{crc:08X}  (CRC of first 8 bytes)\n");
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    // ── Example 4: Unknown algorithm returns a clear error ───────────────

    let dsl_unknown = r#"
        @endian = little;
        struct header @packed {
            crc: u32 = @crc("md5", image);
        }
    "#;

    match generate(dsl_unknown, &HashMap::new(), &sections) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("=== Example 4: Unknown algorithm error ===");
            println!("Got expected error: [{:?}] {}", e.code, e.message);
        }
    }
}
