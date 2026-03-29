//! # Delbin
//!
//! Descriptive Language for Binary Object
//!
//! Delbin is a Domain Specific Language (DSL) and its supporting library for describing
//! and generating binary data structures, primarily used for Header information generation
//! in embedded firmware packaging scenarios.
//!
//! ## Example
//!
//! ```rust
//! use delbin::{generate, Value};
//! use std::collections::HashMap;
//!
//! let dsl = r#"
//!     @endian = little;
//!     struct header @packed {
//!         magic: [u8; 4] = @bytes("FPK\0");
//!         version: u32 = ${VERSION};
//!         size: u32 = @sizeof(image);
//!     }
//! "#;
//!
//! let mut env = HashMap::new();
//! env.insert("VERSION".to_string(), Value::U64(0x0100));
//!
//! let mut sections = HashMap::new();
//! sections.insert("image".to_string(), vec![0u8; 1024]);
//!
//! let result = generate(dsl, &env, &sections).unwrap();
//! assert_eq!(result.data.len(), 12); // 4 + 4 + 4
//! ```

pub mod ast;
pub mod builtin;
pub mod error;
pub mod eval;
pub mod parser;
pub mod types;
pub mod utils;

pub use error::{DelbinError, DelbinWarning, ErrorCode, Result, WarningCode};
pub use types::{Endian, ScalarType, Value};
pub use utils::{
    create_env, create_sections, env_insert_int, env_insert_str, from_hex_string, hex_dump,
    to_hex_string,
};

use std::collections::HashMap;

/// Generation result
#[derive(Debug)]
pub struct GenerateResult {
    /// Generated binary data
    pub data: Vec<u8>,
    /// Warning list
    pub warnings: Vec<DelbinWarning>,
}

/// Generate binary data according to DSL definition
///
/// # Parameters
///
/// * `dsl` - DSL description text
/// * `env` - Environment variable mapping
/// * `sections` - External section data mapping
///
/// # Returns
///
/// Generated binary data and warning list
///
/// # Example
///
/// ```rust
/// use delbin::{generate, Value};
/// use std::collections::HashMap;
///
/// let dsl = r#"
///     @endian = little;
///     struct header @packed {
///         magic: [u8; 4] = @bytes("TEST");
///     }
/// "#;
///
/// let env = HashMap::new();
/// let sections = HashMap::new();
///
/// let result = generate(dsl, &env, &sections).unwrap();
/// assert_eq!(&result.data[..4], b"TEST");
/// ```
pub fn generate(
    dsl: &str,
    env: &HashMap<String, Value>,
    sections: &HashMap<String, Vec<u8>>,
) -> Result<GenerateResult> {
    // Parse DSL
    let file = parser::parse(dsl)?;

    // Evaluate
    let mut evaluator = eval::Evaluator::new(env.clone(), sections.clone());
    let data = evaluator.eval(&file)?;

    Ok(GenerateResult {
        data,
        warnings: evaluator.warnings().to_vec(),
    })
}

/// Generate hexadecimal string
///
/// # Parameters
///
/// * `dsl` - DSL description text
/// * `env` - Environment variable mapping
/// * `sections` - External section data mapping
///
/// # Returns
///
/// Hexadecimal string (uppercase, no separator)
pub fn generate_hex(
    dsl: &str,
    env: &HashMap<String, Value>,
    sections: &HashMap<String, Vec<u8>>,
) -> Result<String> {
    let result = generate(dsl, env, sections)?;
    Ok(to_hex_string(&result.data))
}

/// Validate DSL without generating output
///
/// Checks syntax and semantics. Returns warnings on success, error on failure.
pub fn validate(
    dsl: &str,
    env: &HashMap<String, Value>,
) -> Result<Vec<DelbinWarning>> {
    let file = parser::parse(dsl)?;
    let mut evaluator = eval::Evaluator::new(env.clone(), HashMap::new());
    evaluator.eval(&file)?;
    Ok(evaluator.warnings().to_vec())
}

/// Parse binary data according to DSL field layout
///
/// Reverse of `generate()`. Extracts named field values from raw binary bytes.
///
/// # Parameters
///
/// * `dsl` - DSL description text
/// * `env` - Environment variable mapping (needed to resolve dynamic sizes)
/// * `data` - Raw binary bytes to parse
///
/// # Returns
///
/// Map of field name → value
pub fn parse(
    dsl: &str,
    env: &HashMap<String, Value>,
    data: &[u8],
) -> Result<HashMap<String, Value>> {
    let file = parser::parse(dsl)?;
    let mut evaluator = eval::Evaluator::new(env.clone(), HashMap::new());
    evaluator.parse_bytes(&file, data)
}


/// # Parameters
///
/// * `dsl` - DSL description text
/// * `env` - Environment variable mapping
/// * `image_data` - Target image data
///
/// # Returns
///
/// Merged data (header + image)
pub fn merge(
    dsl: &str,
    env: &HashMap<String, Value>,
    image_data: &[u8],
) -> Result<GenerateResult> {
    let mut sections = HashMap::new();
    sections.insert("image".to_string(), image_data.to_vec());

    let result = generate(dsl, env, &sections)?;

    // Merge header and image
    let mut merged = result.data;
    merged.extend_from_slice(image_data);

    Ok(GenerateResult {
        data: merged,
        warnings: result.warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                magic: [u8; 4] = @bytes("fpk\0");
                version: u32 = 0x0100;
            }
        "#;

        let env = HashMap::new();
        let sections = HashMap::new();

        let result = generate(dsl, &env, &sections).unwrap();
        assert_eq!(result.data.len(), 8);
        assert_eq!(&result.data[..4], b"fpk\0");
        assert_eq!(&result.data[4..8], &[0x00, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn test_generate_with_env() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                version: u32 = (${MAJOR} << 24) | (${MINOR} << 16) | ${PATCH};
            }
        "#;

        let mut env = HashMap::new();
        env.insert("MAJOR".to_string(), Value::U64(1));
        env.insert("MINOR".to_string(), Value::U64(2));
        env.insert("PATCH".to_string(), Value::U64(3));

        let sections = HashMap::new();

        let result = generate(dsl, &env, &sections).unwrap();
        assert_eq!(result.data, vec![0x03, 0x00, 0x02, 0x01]);
    }

    #[test]
    fn test_generate_with_sizeof() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                img_size: u32 = @sizeof(image);
            }
        "#;

        let env = HashMap::new();
        let mut sections = HashMap::new();
        sections.insert("image".to_string(), vec![0u8; 1024]);

        let result = generate(dsl, &env, &sections).unwrap();
        assert_eq!(result.data, vec![0x00, 0x04, 0x00, 0x00]); // 1024 = 0x400
    }

    #[test]
    fn test_generate_with_crc32() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                crc: u32 = @crc32(image);
            }
        "#;

        let env = HashMap::new();
        let mut sections = HashMap::new();
        sections.insert("image".to_string(), b"hello world".to_vec());

        let result = generate(dsl, &env, &sections).unwrap();
        // CRC32 of "hello world" = 0x0D4A1185
        assert_eq!(result.data, vec![0x85, 0x11, 0x4A, 0x0D]);
    }

    #[test]
    fn test_generate_with_self_sizeof() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                magic: [u8; 4] = @bytes("TEST");
                header_size: u32 = @sizeof(@self);
            }
        "#;

        let env = HashMap::new();
        let sections = HashMap::new();

        let result = generate(dsl, &env, &sections).unwrap();
        assert_eq!(result.data.len(), 8);
        // header_size = 8
        assert_eq!(&result.data[4..8], &[0x08, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_generate_with_padding() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                magic: [u8; 4] = @bytes("TEST");
                _pad: [u8; 64 - @offsetof(_pad)];
            }
        "#;

        let env = HashMap::new();
        let sections = HashMap::new();

        let result = generate(dsl, &env, &sections).unwrap();
        assert_eq!(result.data.len(), 64);
    }

    #[test]
    fn test_generate_full_header() {
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

        let mut env = HashMap::new();
        env.insert("VERSION_MAJOR".to_string(), Value::U64(1));
        env.insert("VERSION_MINOR".to_string(), Value::U64(2));
        env.insert("VERSION_PATCH".to_string(), Value::U64(3));
        env.insert("BUILD_NUMBER".to_string(), Value::U64(100));
        env.insert("VERSION_STRING".to_string(), Value::String("1.2.3".to_string()));
        env.insert("UNIX_STAMP".to_string(), Value::U64(1705574400));

        let mut sections = HashMap::new();
        sections.insert("image".to_string(), vec![0xABu8; 1024]);

        let result = generate(dsl, &env, &sections).unwrap();

        // Verify total size
        assert_eq!(result.data.len(), 256);

        // Verify magic
        assert_eq!(&result.data[0..4], b"fpk\0");

        // Verify header_size (offset 10-11)
        assert_eq!(result.data[10], 0x00); // 256 & 0xFF = 0
        assert_eq!(result.data[11], 0x01); // 256 >> 8 = 1

        println!("Generated header ({} bytes):", result.data.len());
        println!("{}", hex_dump(&result.data, 16));
    }

    // ── Type-checking tests ────────────────────────────────────────────

    #[test]
    fn test_string_direct_assign_to_array_is_error() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                magic: [u8; 4] = "bad";
            }
        "#;
        let result = generate(dsl, &HashMap::new(), &HashMap::new());
        assert!(result.is_err(), "expected error for string literal directly assigned to array");
        let msg = result.unwrap_err().message;
        assert!(msg.contains("@bytes"), "error should mention @bytes, got: {}", msg);
    }

    #[test]
    fn test_bytes_to_non_u8_array_is_error() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                data: [u16; 2] = @bytes("AB");
            }
        "#;
        let result = generate(dsl, &HashMap::new(), &HashMap::new());
        assert!(result.is_err(), "expected error for @bytes() on non-u8 array");
        let msg = result.unwrap_err().message;
        assert!(msg.contains("u8"), "error should mention u8, got: {}", msg);
    }

    #[test]
    fn test_integer_truncation_emits_warning() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                small: u8 = 0x1FF;
            }
        "#;
        let result = generate(dsl, &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(result.data, vec![0xFF]); // truncated
        assert!(!result.warnings.is_empty(), "expected truncation warning");
    }

    // ── Range expression tests (P1) ────────────────────────────────────

    #[test]
    fn test_range_field_to_end() {
        // @crc32(@self[magic..]) — from the 'magic' field to end of struct
        let dsl = r#"
            @endian = little;
            struct header @packed {
                magic:  [u8; 4] = @bytes("TEST");
                crc:    u32     = @crc32(@self[magic..]);
            }
        "#;
        let env = HashMap::new();
        let sections = HashMap::new();
        let result = generate(dsl, &env, &sections).unwrap();
        assert_eq!(result.data.len(), 8);
        // Verify CRC is non-zero and matches manual calculation
        let crc_bytes = &result.data[4..8];
        assert_ne!(crc_bytes, &[0u8; 4], "CRC should not be zero");
    }

    #[test]
    fn test_range_field_to_field() {
        // @crc32(@self[magic..body_crc]) — two-field range
        let dsl = r#"
            @endian = little;
            struct header @packed {
                magic:    [u8; 4] = @bytes("TEST");
                reserved: u32     = 0;
                body_crc: u32     = @crc32(@self[magic..body_crc]);
            }
        "#;
        let env = HashMap::new();
        let sections = HashMap::new();
        let result = generate(dsl, &env, &sections).unwrap();
        assert_eq!(result.data.len(), 12);
        let crc_bytes = &result.data[8..12];
        assert_ne!(crc_bytes, &[0u8; 4], "CRC should not be zero");
    }

    // ── P1: env var / shift overflow / @crc unified ────────────────────

    #[test]
    fn test_undefined_env_var_is_error() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                ver: u8 = ${MISSING_VAR};
            }
        "#;
        let result = generate(dsl, &HashMap::new(), &HashMap::new());
        assert!(result.is_err(), "expected Err for undefined env var");
        assert_eq!(result.unwrap_err().code, ErrorCode::E02001);
    }

    #[test]
    fn test_shift_by_64_emits_warning_and_returns_zero() {
        // 1 << 64 cannot fit in u64; should warn W04001 and produce 0
        let dsl = r#"
            @endian = little;
            struct header @packed {
                val: u64 = 1 << 64;
            }
        "#;
        let result = generate(dsl, &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(result.data, vec![0u8; 8], "result should be 0 when shift >= 64");
        assert!(
            result.warnings.iter().any(|w| w.code == WarningCode::W04001),
            "expected W04001 ShiftOverflow warning"
        );
    }

    #[test]
    fn test_crc_unified_equals_crc32() {
        // @crc("crc32", @self[..]) should produce the same bytes as @crc32(@self[..])
        let env = HashMap::new();
        let sects = HashMap::new();

        let dsl_unified = r#"
            @endian = little;
            struct header @packed {
                magic: [u8; 4] = @bytes("TEST");
                crc:   u32     = @crc("crc32", @self[magic..crc]);
            }
        "#;
        let dsl_legacy = r#"
            @endian = little;
            struct header @packed {
                magic: [u8; 4] = @bytes("TEST");
                crc:   u32     = @crc32(@self[magic..crc]);
            }
        "#;

        let unified = generate(dsl_unified, &env, &sects).unwrap();
        let legacy  = generate(dsl_legacy,  &env, &sects).unwrap();
        assert_eq!(unified.data, legacy.data, "@crc(\"crc32\",...) must equal @crc32(...)");
    }

    #[test]
    fn test_crc_unified_crc16_modbus() {
        let mut sections = HashMap::new();
        sections.insert("fw".to_string(), vec![0x01u8, 0x02, 0x03, 0x04]);

        let dsl = r#"
            @endian = little;
            struct header @packed {
                crc16: u16 = @crc("crc16-modbus", fw);
            }
        "#;
        let result = generate(dsl, &HashMap::new(), &sections).unwrap();
        assert_eq!(result.data.len(), 2);
        let crc = u16::from_le_bytes([result.data[0], result.data[1]]);
        assert_ne!(crc, 0, "CRC16-MODBUS should not be zero for non-empty input");
    }

    #[test]
    fn test_crc_unknown_algorithm_is_error() {
        let mut sections = HashMap::new();
        sections.insert("fw".to_string(), vec![0xAAu8]);

        let dsl = r#"
            @endian = little;
            struct header @packed {
                crc: u32 = @crc("nonexistent-algo", fw);
            }
        "#;
        let result = generate(dsl, &HashMap::new(), &sections);
        assert!(result.is_err(), "unknown CRC algorithm should return Err");
        assert_eq!(result.unwrap_err().code, ErrorCode::E04003);
    }

    // ── P2: @align(n) padding ───────────────────────────────────────────

    #[test]
    fn test_align_4_pads_to_boundary() {
        // u8(1) + u16(2) = 3 bytes raw → padded to 4 with @align(4)
        let dsl = r#"
            @endian = little;
            struct header @align(4) {
                tag: u8  = 0xAB;
                val: u16 = 0x1234;
            }
        "#;
        let result = generate(dsl, &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(result.data.len(), 4, "aligned struct should be 4 bytes");
        assert_eq!(result.data[0], 0xAB);
        assert_eq!(result.data[1], 0x34); // little-endian low byte
        assert_eq!(result.data[2], 0x12); // little-endian high byte
        assert_eq!(result.data[3], 0x00); // padding
    }

    #[test]
    fn test_align_already_aligned_no_extra_padding() {
        // u32(4) = 4 bytes raw → already aligned to 4, no padding
        let dsl = r#"
            @endian = little;
            struct header @align(4) {
                val: u32 = 0xDEADBEEF;
            }
        "#;
        let result = generate(dsl, &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(result.data.len(), 4);
    }

    // ── P3: validate() API ─────────────────────────────────────────────

    #[test]
    fn test_validate_valid_dsl_returns_ok() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                version: u8 = 1;
            }
        "#;
        let result = validate(dsl, &HashMap::new());
        assert!(result.is_ok(), "valid DSL should pass validate()");
    }

    #[test]
    fn test_validate_invalid_syntax_returns_error() {
        let result = validate("this is not valid dsl", &HashMap::new());
        assert!(result.is_err(), "invalid syntax should fail validate()");
    }

    #[test]
    fn test_validate_undefined_env_var_returns_error() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                ver: u8 = ${NO_SUCH_VAR};
            }
        "#;
        let result = validate(dsl, &HashMap::new());
        assert!(result.is_err(), "undefined env var should fail validate()");
        assert_eq!(result.unwrap_err().code, ErrorCode::E02001);
    }

    #[test]
    fn test_validate_returns_warnings_for_truncation() {
        let dsl = r#"
            @endian = little;
            struct header @packed {
                small: u8 = 0x1FF;
            }
        "#;
        let warnings = validate(dsl, &HashMap::new()).unwrap();
        assert!(!warnings.is_empty(), "truncation should produce a warning");
        assert!(warnings.iter().any(|w| w.code == WarningCode::W03002));
    }

    // ── P3: parse() API ────────────────────────────────────────────────

    #[test]
    fn test_parse_scalar_fields_little_endian() {
        let dsl = "@endian = little; struct h @packed { ver: u8; flags: u16; size: u32; }";
        let data: &[u8] = &[0x01, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12];
        let result = parse(dsl, &HashMap::new(), data).unwrap();
        assert_eq!(result["ver"].as_u64().unwrap(), 0x01);
        assert_eq!(result["flags"].as_u64().unwrap(), 0x1234);
        assert_eq!(result["size"].as_u64().unwrap(), 0x12345678);
    }

    #[test]
    fn test_parse_scalar_fields_big_endian() {
        let dsl = "@endian = big; struct h @packed { val: u32; }";
        let data: &[u8] = &[0x12, 0x34, 0x56, 0x78];
        let result = parse(dsl, &HashMap::new(), data).unwrap();
        assert_eq!(result["val"].as_u64().unwrap(), 0x12345678);
    }

    #[test]
    fn test_parse_array_field_returns_bytes() {
        let dsl = "@endian = little; struct h @packed { magic: [u8; 4]; }";
        let data: &[u8] = b"TEST";
        let result = parse(dsl, &HashMap::new(), data).unwrap();
        assert_eq!(result["magic"].as_bytes().unwrap(), b"TEST");
    }

    #[test]
    fn test_parse_data_too_short_is_error() {
        let dsl = "@endian = little; struct h @packed { size: u32; }";
        let data: &[u8] = &[0x01, 0x02]; // only 2 bytes, needs 4
        let result = parse(dsl, &HashMap::new(), data);
        assert!(result.is_err(), "short data should return Err");
    }

    #[test]
    fn test_parse_roundtrip() {
        let dsl = r#"
            @endian = little;
            struct h @packed {
                version: u8  = 3;
                flags:   u16 = 0x1234;
                size:    u32 = 0xDEADBEEF;
            }
        "#;
        let generated = generate(dsl, &HashMap::new(), &HashMap::new()).unwrap();
        let parsed = parse(dsl, &HashMap::new(), &generated.data).unwrap();
        assert_eq!(parsed["version"].as_u64().unwrap(), 3);
        assert_eq!(parsed["flags"].as_u64().unwrap(), 0x1234);
        assert_eq!(parsed["size"].as_u64().unwrap(), 0xDEAD_BEEF);
    }
}
