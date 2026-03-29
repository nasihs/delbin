# Delbin - Descriptive Language for Binary Objects

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)

A Domain-Specific Language (DSL) and its supporting library for describing and generating binary data structures, primarily designed for embedded firmware header generation.

## Features

Delbin enables firmware engineers to:

- ✅ Define binary data structures using human-readable syntax
- ✅ Automatically calculate sizes, offsets, and checksums
- ✅ Support environment variable substitution
- ✅ Generate binary data from DSL definitions
- ✅ CRC32, CRC16-MODBUS, and SHA256 checksums — unified `@crc("algo", ...)` API
- ✅ Handle self-referencing fields (e.g., header CRC)
- ✅ Full range expressions: `@self`, `@self[..field]`, `@self[field..]`, `@self[field_a..field_b]`
- ✅ Support both little-endian and big-endian byte orders
- ✅ Flexible array initialization with multiple syntax forms
- ✅ Struct alignment padding via `@align(n)`
- ✅ Type safety: hard errors for type mismatches, warnings for value truncation
- ✅ `validate()` API — check DSL without generating bytes
- ✅ `parse()` API — reverse-read binary data according to DSL schema
- ✅ Command-line tool with `--env`, `--section`, `--format`, `--output`, `--verbose`

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

Struct attributes:

| Attribute | Description |
|-----------|-------------|
| `@packed` | No alignment padding between fields |
| `@align(n)` | Pad struct output to next `n`-byte boundary |

```rust
struct header @align(4) {   // output always a multiple of 4 bytes
    tag:  u8  = 0xAB;
    val:  u16 = 0x1234;
    // auto-padded to 4 bytes
}
```

### Types

- **Scalar types**: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`
- **Array types**: `[u8; 4]`, `[u32; N]`

### Array Initialization

Arrays support flexible initialization syntax:

```rust
data: [u8; 4];                    // Default: all zeros
pattern: [u8; 8] = [0xFF; 8];     // Repeat value (explicit count)
fill: [u8; 8] = [0xAA; _];        // Repeat value (inferred count)
values: [u8; 4] = [1, 2, 3, 4];   // Element list (full)
partial: [u8; 8] = [1, 2];        // Element list (partial, pad with 0)
magic: [u8; 4] = @bytes("FPK");   // Function call

// Environment variables in arrays
data: [u8; 4] = [${VAL}; _];      // Repeat with env var
mixed: [u8; 4] = [1, ${X}, 3, 4]; // Element list with env var
```

### Expressions

- **Literals**: `0x1234`, `0b1010`, `42`, `"string"`
- **Environment variables**: `${VAR_NAME}`
- **Operators**: `|`, `&`, `<<`, `>>`, `+`, `-`, `~`

### Built-in Functions

| Function | Description | Example |
|----------|-------------|---------|
| `@bytes(str)` | Convert string to byte array | `@bytes("FPK\0")` |
| `@sizeof(section)` | Get size of section or struct | `@sizeof(image)` |
| `@offsetof(field)` | Get field byte offset | `@offsetof(crc)` |
| `@crc32(range)` | CRC32-ISO-HDLC (alias for `@crc("crc32", ...)`) | `@crc32(image)` |
| `@crc("algo", range)` | CRC with named algorithm | `@crc("crc16-modbus", image)` |
| `@sha256(range)` | SHA256 hash (returns `[u8; 32]`) | `@sha256(image)` |

**Supported CRC algorithms** for `@crc()`:

| Name | Width | Description |
|------|-------|-------------|
| `"crc32"` / `"crc32-iso-hdlc"` | 32-bit | Same as `@crc32()` |
| `"crc16-modbus"` | 16-bit | CRC16-MODBUS |

### Range Expressions

```rust
@self                       // Entire current struct
@self[..field]              // From start to before field
@self[field..]              // From field to end of struct
@self[field_a..field_b]     // From field_a to before field_b
@self[0x10..field]          // From byte offset 0x10 to before field
section_name                // Entire external section (e.g. image)
```

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
    
    // Self-referencing: CRC of body from magic to body_crc (partial range)
    body_crc:       u32 = @crc("crc16-modbus", @self[magic..body_crc]);
    
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
- [x] `@crc("algorithm", range)` unified CRC with `crc32` and `crc16-modbus`
- [x] Self-referencing fields with two-phase evaluation
- [x] Full range expressions: `@self`, `@self[..field]`, `@self[field..]`, `@self[field_a..field_b]`
- [x] Little-endian and big-endian support
- [x] Struct attributes: `@packed`, `@align(n)`
- [x] Array literal initialization with five syntax forms
- [x] Environment variables in array elements
- [x] Type checking: hard error for string→array without `@bytes`, for `@bytes` on non-`u8` arrays
- [x] Value truncation warning (W03002) when value overflows target field width
- [x] Shift overflow warning (W04001) for shift amount ≥ 64
- [x] Structured error and warning codes (E01xxx–E05xxx, W03xxx–W04xxx)
- [x] `validate()` API — parse + semantic check without generating bytes
- [x] `parse()` API — reverse-read binary into named fields
- [x] `merge()` API — generate header and prepend to image in one call
- [x] CLI tool (`delbin`) with `--env`, `--section`, `--format`, `--output`, `--verbose`

### 🚧 Planned Features

- [ ] Multiple structs per DSL file
- [ ] Additional CRC algorithms (currently: `crc32`, `crc16-modbus`)
- [ ] Additional hash algorithms (`@hash()` with algorithm parameter)
- [ ] TOML configuration file support for CLI

### ❌ Not Planned

- Firmware encryption/decryption
- Digital signature generation/verification
- Firmware transmission protocols
- Dynamic TLV structure parsing

## API Reference

### Core Functions

```rust
/// Generate binary output from DSL
pub fn generate(
    dsl: &str,
    env: &HashMap<String, Value>,
    sections: &HashMap<String, Vec<u8>>,
) -> Result<GenerateResult>;

/// Generate and return as uppercase hex string
pub fn generate_hex(
    dsl: &str,
    env: &HashMap<String, Value>,
    sections: &HashMap<String, Vec<u8>>,
) -> Result<String>;

/// Generate header and prepend to image_data
pub fn merge(
    dsl: &str,
    env: &HashMap<String, Value>,
    image_data: &[u8],
) -> Result<GenerateResult>;

/// Validate DSL syntax and semantics without generating output.
/// Returns any warnings on success.
pub fn validate(
    dsl: &str,
    env: &HashMap<String, Value>,
) -> Result<Vec<DelbinWarning>>;

/// Parse raw bytes back into named field values according to the DSL layout.
pub fn parse(
    dsl: &str,
    env: &HashMap<String, Value>,
    data: &[u8],
) -> Result<HashMap<String, Value>>;
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
    pub warnings: Vec<DelbinWarning>,
}
```

## CLI

```
delbin [OPTIONS] <INPUT>

Arguments:
  <INPUT>         DSL file path; use '-' to read from stdin

Options:
  -o, --output <FILE>        Write to file instead of stdout
      --format <hex|bin>     Output format: 'hex' (default) or 'bin' (raw bytes)
      --env <KEY=VALUE>      Set environment variable (repeatable)
      --section <NAME=FILE>  Load section data from file (repeatable)
      --verbose              Print warnings to stderr
  -h, --help
  -V, --version
```

**CLI examples:**

```bash
# Print hex to stdout
delbin header.dsl

# Write binary file
delbin header.dsl --format bin -o header.bin

# Inject environment variables
delbin header.dsl --env VERSION=256 --env BUILD_ID=42

# Pass external section (for @sizeof / @crc32)
delbin header.dsl --section image=firmware.bin --format bin -o header.bin

# Read DSL from stdin (useful in CI pipelines)
cat header.dsl | delbin - --env VERSION=1

# Print truncation / overflow warnings
delbin header.dsl --verbose
```

## Error Handling

Delbin uses structured error and warning codes:

| Category | Code Range | Description |
|----------|------------|-------------|
| Parse errors | E01xxx | DSL syntax errors |
| Semantic errors | E02xxx | Undefined variables/fields/sections |
| Type errors | E03xxx | Type mismatches, size mismatches |
| Evaluation errors | E04xxx | Expression evaluation failures |
| IO errors | E05xxx | File operation errors |
| String warnings | W03001 | String truncated to fit array |
| Truncation warnings | W03002 | Integer value truncated to fit field width |
| Shift warnings | W04001 | Shift amount ≥ 64 bits (result is 0) |

Example:
```rust
match generate(dsl, &env, &sections) {
    Ok(result) => {
        for w in &result.warnings {
            eprintln!("[{:?}] {}", w.code, w.message);
        }
    },
    Err(e) => eprintln!("[{}] {}", e.code, e.message),
}
```

## Examples

```bash
# Firmware header with CRC and SHA256
cargo run --example basic

# All array initialization syntax forms
cargo run --example array_syntax

# Unified @crc() and CRC16-MODBUS
cargo run --example crc_algorithms

# validate() and parse() APIs
cargo run --example validate_and_parse
```

## Testing

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

