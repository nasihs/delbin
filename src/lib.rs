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

/// Merge header into target file
///
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
}
