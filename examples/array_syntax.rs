//! Comprehensive array initialization syntax examples

use delbin::{generate, hex_dump, Value};
use std::collections::HashMap;

fn main() {
    let dsl = r#"
        @endian = little;

        struct demo @packed {
            // Syntax 1: Default zero fill
            zeros: [u8; 4];
            
            // Syntax 2: Fill with value (full form)
            pattern1: [u8; 8] = [0xFF; 8];
            
            // Syntax 3: Fill with value (inferred form)
            pattern2: [u8; 8] = [0xAA; _];
            
            // Syntax 4: Element list
            bytes1: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
            
            // Syntax 4: Element list with partial fill (rest filled with 0)
            bytes2: [u8; 8] = [0x11, 0x22];
            
            // Syntax 5: Function call
            magic: [u8; 8] = @bytes("DELBIN");
            
            // Mixed types
            u16_array: [u16; 4] = [0x1234; _];
            u32_values: [u32; 2] = [0xDEADBEEF, 0xCAFEBABE];
        }
    "#;

    let env = HashMap::new();
    let sections = HashMap::new();

    match generate(dsl, &env, &sections) {
        Ok(result) => {
            println!("Generated binary data ({} bytes):\n", result.data.len());
            println!("{}", hex_dump(&result.data, 16));
            
            if !result.warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &result.warnings {
                    println!("  [{:?}] {}", warning.code, warning.message);
                }
            }
            
            println!("\nField breakdown:");
            println!("  zeros:      4 bytes  - all 0x00");
            println!("  pattern1:   8 bytes  - all 0xFF");
            println!("  pattern2:   8 bytes  - all 0xAA");
            println!("  bytes1:     4 bytes  - 0x01, 0x02, 0x03, 0x04");
            println!("  bytes2:     8 bytes  - 0x11, 0x22, then 6x 0x00");
            println!("  magic:      8 bytes  - 'DELBIN' + 2x 0x00");
            println!("  u16_array:  8 bytes  - 4x 0x1234 (little-endian)");
            println!("  u32_values: 8 bytes  - 0xDEADBEEF, 0xCAFEBABE (little-endian)");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
