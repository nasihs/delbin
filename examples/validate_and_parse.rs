//! `validate()` and `parse()` API examples
//!
//! `validate()` — check DSL syntax and semantics without producing bytes.
//! `parse()`    — reverse-read raw binary bytes into named field values.

use delbin::{generate, parse, validate, Value, WarningCode};
use std::collections::HashMap;

fn main() {
    // ── Example 1: validate() catches errors early ───────────────────────

    println!("=== Example 1: validate() ===");

    let bad_dsl = r#"
        @endian = little;
        struct header @packed {
            version: u8 = ${MISSING_VAR};
        }
    "#;

    match validate(bad_dsl, &HashMap::new()) {
        Ok(_) => println!("Unexpected OK"),
        Err(e) => println!("Caught error: [{:?}] {}", e.code, e.message),
    }

    // validate() returns warnings (not errors) for non-fatal issues
    let warn_dsl = r#"
        @endian = little;
        struct header @packed {
            small: u8 = 0x1FF;
        }
    "#;

    match validate(warn_dsl, &HashMap::new()) {
        Ok(warnings) => {
            println!("\nvalidate() succeeded with {} warning(s):", warnings.len());
            for w in &warnings {
                println!("  [{:?}] {}", w.code, w.message);
            }
            assert!(warnings.iter().any(|w| w.code == WarningCode::W03002));
        }
        Err(e) => eprintln!("Unexpected error: {e}"),
    }

    // ── Example 2: parse() reads named fields from binary ────────────────

    println!("\n=== Example 2: parse() ===");

    let dsl = r#"
        @endian = little;
        struct header @packed {
            magic:   [u8; 4];
            version: u32;
            flags:   u16;
        }
    "#;

    // Build some binary data manually
    let mut data = Vec::new();
    data.extend_from_slice(b"FPK\0");             // magic
    data.extend_from_slice(&0x0001_0203u32.to_le_bytes()); // version
    data.extend_from_slice(&0xABCDu16.to_le_bytes());      // flags

    match parse(dsl, &HashMap::new(), &data) {
        Ok(fields) => {
            println!("Parsed {} fields:", fields.len());
            println!("  magic   = {:?}", fields["magic"].as_bytes().unwrap());
            println!("  version = 0x{:08X}", fields["version"].as_u64().unwrap());
            println!("  flags   = 0x{:04X}", fields["flags"].as_u64().unwrap());
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    // ── Example 3: generate → parse round-trip ───────────────────────────

    println!("\n=== Example 3: generate → parse round-trip ===");

    let roundtrip_dsl = r#"
        @endian = little;
        struct header @packed {
            magic:   [u8; 4] = @bytes("DLBN");
            version: u32     = ${VERSION};
            size:    u32     = @sizeof(@self);
        }
    "#;

    let mut env = HashMap::new();
    env.insert("VERSION".to_string(), Value::U64(0x0102_0304));

    let generated = generate(roundtrip_dsl, &env, &HashMap::new()).unwrap();
    println!("Generated {} bytes", generated.data.len());

    let parsed = parse(roundtrip_dsl, &env, &generated.data).unwrap();
    println!("Round-trip results:");
    println!("  magic   = {:?}", parsed["magic"].as_bytes().unwrap());
    println!("  version = 0x{:08X}", parsed["version"].as_u64().unwrap());
    println!("  size    = {}", parsed["size"].as_u64().unwrap());

    assert_eq!(parsed["version"].as_u64().unwrap(), 0x0102_0304);
    assert_eq!(parsed["size"].as_u64().unwrap() as usize, generated.data.len());
    println!("✓ Round-trip verified");

    // ── Example 4: parse() error when data is too short ──────────────────

    println!("\n=== Example 4: parse() — data too short ===");

    let short_dsl = "@endian = little; struct h @packed { val: u32; }";
    match parse(short_dsl, &HashMap::new(), &[0x01, 0x02]) {
        Ok(_) => println!("Unexpected OK"),
        Err(e) => println!("Caught expected error: [{:?}] {}", e.code, e.message),
    }
}
