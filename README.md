# Delbin

**Descriptive Language for Binary Object**

A Domain-Specific Language (DSL) and its supporting library for describing and generating binary data structures, primarily designed for embedded firmware header generation.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)

## Features

Delbin enables firmware engineers to:

- ✅ Define binary data structures using human-readable syntax
- ✅ Automatically calculate sizes, offsets, and checksums
- ✅ Support environment variable substitution
- ✅ Generate binary data from DSL definitions
- ✅ Support CRC32 and SHA256 checksums
- ✅ Handle self-referencing fields (e.g., header CRC)
- ✅ Support both little-endian and big-endian byte orders

## Quick Start

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
delbin = "0.1"
```

### Basic Usage

```rust
use delbin::{generate, Value};
use std::collections::HashMap;

let dsl = r#"
    @endian = little;
    
    struct header @packed {
        magic:       [u8; 4] = @bytes("FPK\0");
        version:     u32 = ${VERSION};
        img_size:    u32 = @sizeof(image);
        img_crc:     u32 = @crc32(image);
    }
"#;

// Set environment variables
let mut env = HashMap::new();
env.insert("VERSION".to_string(), Value::U64(0x0100));

// Set sections (external binary data)
let mut sections = HashMap::new();
sections.insert("image".to_string(), vec![0u8; 1024]);

// Generate binary data
let result = generate(dsl, &env, &sections)?;
println!("Generated {} bytes", result.data.len());
```

## DSL Syntax Overview

### Global Directives

```rust
@endian = little;  // or big
```

### Struct Definition

```rust
struct header @packed {
    field_name: type = expression;
}
```

### Types

- **Scalar types**: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`
- **Array types**: `[u8; 4]`, `[u32; N]`

### Expressions

- **Literals**: `0x1234`, `0b1010`, `42`, `"string"`
- **Environment variables**: `${VAR_NAME}`
- **Operators**: `|`, `&`, `<<`, `>>`, `+`, `-`, `~`

### Built-in Functions

| Function | Description | Example |
|----------|-------------|---------|
| `@bytes(str)` | Convert string to byte array | `@bytes("FPK\0")` |
| `@sizeof(section)` | Get size of section or struct | `@sizeof(image)` |
| `@offsetof(field)` | Get field offset | `@offsetof(crc)` |
| `@crc32(range)` | Calculate CRC32 checksum | `@crc32(image)` |
| `@sha256(range)` | Calculate SHA256 hash | `@sha256(image)` |

### Range Expressions

```rust
@self              // Entire current struct
@self[..field]     // From start to field
@self[field..]     // From field to end
```

For complete syntax documentation, see [GRAMMAR.md](GRAMMAR.md).

## Example: Firmware Header

```text
@endian = little;

struct header @packed {
    // Magic number
    magic:          [u8; 4] = @bytes("fpk\0");
    
    // Version (packed: major.minor.patch)
    fw_version:     u32 = (${VERSION_MAJOR} << 24) | 
                          (${VERSION_MINOR} << 16) | 
                          ${VERSION_PATCH};
    
    // Sizes
    header_size:    u32 = @sizeof(@self);
    img_size:       u32 = @sizeof(image);
    
    // Timestamp
    timestamp:      u32 = ${UNIX_STAMP};
    
    // Checksums
    img_crc32:      u32 = @crc32(image);
    img_sha256:     [u8; 32] = @sha256(image);
    
    // Self-referencing header CRC
    header_crc32:   u32 = @crc32(@self[..header_crc32]);
    
    // Padding to 256 bytes
    _padding:       [u8; 256 - @offsetof(_padding)];
}
```

## Implementation Status

### ✅ Implemented Features

- [x] DSL parser with Pest grammar
- [x] AST generation
- [x] Binary data generation
- [x] Environment variable substitution
- [x] Built-in functions: `@bytes`, `@sizeof`, `@offsetof`, `@crc32`, `@sha256`
- [x] Self-referencing fields support
- [x] Range expressions
- [x] Little-endian and big-endian support
- [x] Struct attributes (`@packed`)
- [x] Error reporting with error codes
- [x] Warning system

### 🚧 Planned Features

- [ ] Data parsing (read binary according to DSL schema)
- [ ] Data validation (verify binary data integrity)
- [ ] `@align(n)` attribute support
- [ ] Additional CRC algorithms (`@crc16`, `@crc()` with algorithm parameter)
- [ ] Additional hash algorithms (`@hash()` with algorithm parameter)
- [ ] File merge API
- [ ] TOML configuration file support
- [ ] CLI tool

### ❌ Not Planned

- Firmware encryption/decryption
- Digital signature generation/verification
- Firmware transmission protocols
- Dynamic TLV structure parsing

## API Reference

### Core Functions

```rust
pub fn generate(
    dsl: &str,
    env: &HashMap<String, Value>,
    sections: &HashMap<String, Vec<u8>>,
) -> Result<GenerateResult>;

pub fn generate_hex(
    dsl: &str,
    env: &HashMap<String, Value>,
    sections: &HashMap<String, Vec<u8>>,
) -> Result<String>;

pub fn merge(
    dsl: &str,
    env: &HashMap<String, Value>,
    image_data: &[u8],
) -> Result<GenerateResult>;
```

### Types

```rust
pub enum Value {
    U8(u8), U16(u16), U32(u32), U64(u64),
    I8(i8), I16(i16), I32(i32), I64(i64),
    Bytes(Vec<u8>),
    String(String),
}

pub struct GenerateResult {
    pub data: Vec<u8>,
    pub warnings: Vec<DelBinWarning>,
}
```

## Error Handling

Delbin uses structured error codes following IEEE 29148 standard:

| Category | Code Range | Description |
|----------|------------|-------------|
| Parse errors | E01xxx | DSL syntax errors |
| Semantic errors | E02xxx | Undefined variables/fields |
| Type errors | E03xxx | Type mismatches |
| Evaluation errors | E04xxx | Expression evaluation failures |
| IO errors | E05xxx | File operation errors |

Example:
```rust
match generate(dsl, &env, &sections) {
    Ok(result) => { /* use result */ },
    Err(e) => eprintln!("[{}] {}", e.code, e.message),
}
```

## Examples

Run the basic example:

```bash
cargo run --example basic
```

## Testing

Run all tests:

```bash
cargo test
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Documentation

- [Complete Grammar Reference](GRAMMAR.md)

## References

- [Pest Parser](https://pest.rs) - PEG parser used for DSL parsing
- [MCUboot](https://docs.mcuboot.com) - Inspiration for firmware header formats
- [CRC RevEng Catalogue](https://reveng.sourceforge.io/crc-catalogue) - CRC algorithm reference
