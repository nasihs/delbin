# Delbin Grammar Reference

Complete syntax specification for the Delbin DSL (Descriptive Language for Binary Object).

## Table of Contents

- [File Structure](#file-structure)
- [Directives](#directives)
- [Struct Definition](#struct-definition)
- [Types](#types)
- [Expressions](#expressions)
- [Built-in Functions](#built-in-functions)
- [Range Expressions](#range-expressions)
- [EBNF Grammar](#ebnf-grammar)

## File Structure

A Delbin file consists of optional directives followed by a struct definition:

```
[directives]
struct <name> [attributes] {
    <field_definitions>
}
```

## Directives

### Endianness Directive

Specifies the byte order for multi-byte values:

```rust
@endian = little;  // Little-endian (default)
@endian = big;     // Big-endian
```

- **Default**: `little`
- **Scope**: Entire file
- **Occurrence**: At most once, must appear before struct definition

## Struct Definition

### Basic Syntax

```rust
struct <name> <attributes> {
    <field>: <type> [= <expression>];
    ...
}
```

### Struct Attributes

| Attribute | Syntax | Description |
|-----------|--------|-------------|
| `@packed` | `struct header @packed { ... }` | Compact layout, no padding between fields |
| `@align(n)` | `struct header @align(4) { ... }` | Pad struct output to next `n`-byte boundary |

`@align(n)` rounds the total struct size up to the nearest multiple of `n`. Fields keep their natural layout; padding bytes (0x00) are appended at the end.

```rust
struct config @align(4) {
    tag:  u8  = 0xAB;   // offset 0, 1 byte
    val:  u16 = 0x1234; // offset 1, 2 bytes
    // raw = 3 bytes → padded to 4 bytes, one 0x00 appended
}
```

## Types

### Scalar Types

| Type | Size | Description |
|------|------|-------------|
| `u8` | 1 byte | Unsigned 8-bit integer |
| `u16` | 2 bytes | Unsigned 16-bit integer |
| `u32` | 4 bytes | Unsigned 32-bit integer |
| `u64` | 8 bytes | Unsigned 64-bit integer |
| `i8` | 1 byte | Signed 8-bit integer |
| `i16` | 2 bytes | Signed 16-bit integer |
| `i32` | 4 bytes | Signed 32-bit integer |
| `i64` | 8 bytes | Signed 64-bit integer |

### Array Types

Arrays use Rust-style syntax:

```rust
[<scalar_type>; <length>]
```

Examples:
```rust
magic: [u8; 4];              // 4-byte array
padding: [u8; 32];           // 32-byte array
data: [u32; 8];              // Array of 8 u32 elements
dynamic: [u8; 128 - @offsetof(_pad)];  // Computed length
```

### Array Initialization

Arrays support five initialization syntax forms:

| Syntax | Description | Behavior |
|--------|-------------|----------|
| `[u8; N]` | Default initialization | Fills all elements with `0x00` |
| `[u8; N] = [val; N]` | Repeat value (explicit count) | Fills N elements with `val` |
| `[u8; N] = [val; _]` | Repeat value (inferred count) | Fills all elements with `val` (count inferred from type) |
| `[u8; N] = [a, b, c]` | Element list | Uses specified values, pads remaining with `0x00` |
| `[u8; N] = @bytes("str")` | Function call | Uses function return value |

#### Detailed Behavior

**Repeat Form: `[val; count]`**
- If `count < N`: fills `count` elements with `val`, pads remaining with `0x00`
- If `count == N`: fills all elements with `val`
- If `count > N`: fills `N` elements (truncates), emits warning W03002
- `count` can be `_` to infer from array type length

**Element List Form: `[a, b, c, ...]`**
- If fewer elements than N: pads remaining with `0x00`
- If more elements than N: truncates and emits warning W03002
- Elements can be literals or environment variables: `[1, ${VAR}, 3]`

#### Examples

```rust
// Default initialization
zeros: [u8; 4];                      // [0x00, 0x00, 0x00, 0x00]

// Repeat with explicit count
pattern1: [u8; 4] = [0xFF; 4];       // [0xFF, 0xFF, 0xFF, 0xFF]
pattern2: [u8; 4] = [0xFF; 2];       // [0xFF, 0xFF, 0x00, 0x00] - partial fill

// Repeat with inferred count
fill: [u8; 4] = [0xAA; _];           // [0xAA, 0xAA, 0xAA, 0xAA]

// Element list (full)
values: [u8; 4] = [1, 2, 3, 4];      // [0x01, 0x02, 0x03, 0x04]

// Element list (partial)
partial: [u8; 8] = [0x11, 0x22];     // [0x11, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]

// With environment variables
env_repeat: [u8; 4] = [${VAL}; _];   // Repeat env var value
env_list: [u8; 4] = [1, ${X}, 3, 4]; // Mix literals and env vars

// Multi-byte types
u16_data: [u16; 4] = [0x1234; _];    // Four u16 values (respects endianness)
u32_vals: [u32; 2] = [0xDEAD, 0xBEEF]; // Two u32 values

// Function calls
magic: [u8; 8] = @bytes("DELBIN");   // "DELBIN" + 0x00 padding
```

## Expressions

### Literals

#### Integer Literals

| Format | Syntax | Example |
|--------|--------|---------|
| Decimal | `[0-9]+` | `12345` |
| Hexadecimal | `0x[0-9a-fA-F]+` | `0xDEADBEEF` |
| Binary | `0b[01]+` | `0b10101010` |

#### String Literals

```rust
"hello world"
"fpk\0"           // With null terminator
"line1\nline2"    // Escape sequences
"\x41\x42\x43"    // Hexadecimal escapes
```

Supported escape sequences:
- `\n` - Newline
- `\r` - Carriage return
- `\t` - Tab
- `\\` - Backslash
- `\"` - Double quote
- `\0` - Null character
- `\xHH` - Hexadecimal byte (e.g., `\x41` = 'A')

### Environment Variables

Reference environment variables using `${VAR_NAME}` syntax:

```rust
version: u32 = ${VERSION};
timestamp: u32 = ${UNIX_STAMP};
```

Environment variables must be defined by the calling application before generation.

### Operators

| Operator | Description | Precedence | Example |
|----------|-------------|------------|---------|
| `()` | Grouping | Highest | `(a + b) * c` |
| `~` | Bitwise NOT | High | `~0x0F` |
| `<<` | Left shift | Medium | `1 << 8` |
| `>>` | Right shift | Medium | `0xFF00 >> 8` |
| `&` | Bitwise AND | Low | `flags & 0x01` |
| `\|` | Bitwise OR | Lowest | `FLAG_A \| FLAG_B` |
| `+` | Addition | Medium | `size + 4` |
| `-` | Subtraction | Medium | `256 - offset` |

### Operator Examples

```rust
// Version packing
fw_version: u32 = (${MAJOR} << 24) | (${MINOR} << 16) | ${PATCH};

// Flag combination
flags: u32 = FLAG_SIGNED | FLAG_ENCRYPTED;

// Bit masking
masked: u32 = value & 0xFF00;

// Arithmetic
padding_size: u32 = 256 - @offsetof(_padding);
```

## Built-in Functions

### @bytes()

Convert string to byte array.

```rust
@bytes(<string>)
```

**Parameters:**
- `string`: String literal or environment variable

**Returns:** Byte array

**Behavior:**
- If string is shorter than target array: pad with 0x00
- If string is longer: truncate and emit warning W03001

**Examples:**
```rust
magic: [u8; 4] = @bytes("FPK");          // [0x46, 0x50, 0x4B, 0x00]
partition: [u8; 16] = @bytes("app");     // "app" + 13×0x00
name: [u8; 8] = @bytes(${NAME});         // From environment variable
```

### @sizeof()

Calculate size of section or struct.

```rust
@sizeof(<section>)
@sizeof(@self)
```

**Parameters:**
- `section`: Section name (e.g., `image`)
- `@self`: Current struct

**Returns:** `u32` size in bytes

**Examples:**
```rust
img_size: u32 = @sizeof(image);          // Size of image section
header_size: u32 = @sizeof(@self);       // Size of current struct
total_size: u32 = @sizeof(header) + @sizeof(image);
```

### @offsetof()

Calculate field offset within struct.

```rust
@offsetof(<field_name>)
```

**Parameters:**
- `field_name`: Field name in current struct

**Returns:** `u32` byte offset

**Special case:** Self-referencing (field references itself) returns current offset.

**Examples:**
```rust
crc_offset: u32 = @offsetof(header_crc);     // Offset of header_crc field
_pad: [u8; 128 - @offsetof(_pad)];           // Self-reference for padding
```

### @crc32()

Calculate CRC32 checksum (ISO-HDLC algorithm). Equivalent to `@crc("crc32", ...)`.

```rust
@crc32(<range>)
```

**Algorithm:** CRC32-ISO-HDLC
- Polynomial: `0x04C11DB7` (reflected)
- Initial value / XOR out: `0xFFFFFFFF`

**Returns:** `u32`

**Examples:**
```rust
img_crc: u32 = @crc32(image);                    // CRC of image section
header_crc: u32 = @crc32(@self[..header_crc]);    // Self-referencing CRC
partial: u32 = @crc32(@self[magic..partial]);      // Partial struct range
```

### @crc()

Calculate CRC using a named algorithm.

```rust
@crc(<"algorithm">, <range>)
```

**Parameters:**
- `"algorithm"`: String literal naming the algorithm (see table below)
- `range`: Section reference or range expression

**Returns:** integer width matches algorithm (e.g., `u32` for crc32, `u16` for crc16-modbus)

**Supported algorithms:**

| Algorithm name | Width | Description |
|----------------|-------|-------------|
| `"crc32"` / `"crc32-iso-hdlc"` | 32-bit | CRC32-ISO-HDLC (same as `@crc32()`) |
| `"crc16-modbus"` | 16-bit | CRC16-MODBUS |

**Examples:**
```rust
// Same output as @crc32()
img_crc: u32 = @crc("crc32", image);

// CRC16-MODBUS over entire image
crc16:   u16 = @crc("crc16-modbus", image);

// Self-referencing partial range
body_crc: u32 = @crc("crc32", @self[magic..body_crc]);
```

**Error:** Unknown algorithm name returns `E04003 InvalidArgument`.

### @sha256()

Calculate SHA256 hash.

```rust
@sha256(<range>)
```

**Parameters:**
- `range`: Section reference or range expression

**Returns:** `[u8; 32]` hash value

**Examples:**
```rust
img_hash: [u8; 32] = @sha256(image);         // SHA256 of image section
combined: [u8; 32] = @sha256(header, image); // Multiple sections (⚠️ Not yet implemented)
```

## Range Expressions

Range expressions specify data ranges for checksum/hash calculations.

### Syntax

| Syntax | Description |
|--------|-------------|
| `<section>` | Entire named section (e.g., `image`) |
| `@self` | Entire current struct |
| `@self[..<field>]` | From struct start to before `field` |
| `@self[<field>..]` | From `field` to end of struct |
| `@self[<field_a>..<field_b>]` | From `field_a` to before `field_b` |
| `@self[<offset>..<field>]` | From numeric byte offset to before `field` |

`start` (if given) is the **inclusive** first byte; `end` (if given) is the **exclusive** last byte (i.e., the field at `end` is not included).

### Examples

```rust
// CRC of entire image section
img_crc: u32 = @crc32(image);

// CRC from start to before header_crc (self-referencing)
header_crc: u32 = @crc32(@self[..header_crc]);

// CRC from the 'magic' field to end of struct
tail_crc: u32 = @crc32(@self[magic..]);

// CRC of fields between 'magic' and 'body_crc' (body_crc not included)
body_crc: u32 = @crc32(@self[magic..body_crc]);

// CRC from byte 0x10 to before 'header_crc'
partial_crc: u32 = @crc32(@self[0x10..header_crc]);
```

### Self-Referencing Fields (Two-Phase Evaluation)

When a field computes a checksum over a range that includes bytes written before it (or the struct end), Delbin uses two-phase evaluation:

1. **First pass:** write all non-self-referencing fields normally; fill self-referencing fields with `0x00`
2. **Second pass:** recompute the checksum once all bytes are known; backfill the placeholder

A field is deferred when it calls `@crc32`, `@sha256`, or `@crc` with an `@self` range argument.

**Example:**
```rust
struct header @packed {
    magic: [u8; 4] = @bytes("TEST");
    size: u32 = @sizeof(@self);
    // Deferred: covers all bytes [0 .. offset_of(header_crc)]
    header_crc: u32 = @crc32(@self[..header_crc]);
}
```

## EBNF Grammar

Complete grammar in Extended Backus-Naur Form:

```ebnf
(* Top-level structure *)
file            = { directive } , struct_def ;

(* Global directives *)
directive       = "@" , directive_name , "=" , directive_value , ";" ;
directive_name  = "endian" ;
directive_value = "little" | "big" ;

(* Struct definition *)
struct_def      = "struct" , identifier , { struct_attr } , "{" , { field_def } , "}" ;
struct_attr     = "@packed" | ( "@align" , "(" , dec_number , ")" ) ;

(* Field definition — initializer is either an array literal or a general expression *)
field_def       = identifier , ":" , type_spec , [ "=" , ( array_literal | expression ) ] , ";" ;

(* Types *)
type_spec       = scalar_type | array_type ;
scalar_type     = ( "u" | "i" ) , ( "8" | "16" | "32" | "64" ) ;
array_type      = "[" , scalar_type , ";" , expression , "]" ;

(* Expressions *)
expression      = or_expr ;
or_expr         = and_expr , { "|" , and_expr } ;
and_expr        = shift_expr , { "&" , shift_expr } ;
shift_expr      = add_expr , { ( "<<" | ">>" ) , add_expr } ;
add_expr        = unary_expr , { ( "+" | "-" ) , unary_expr } ;
unary_expr      = [ "~" ] , primary_expr ;
primary_expr    = builtin_call | env_var | hex_number | dec_number | bin_number
                | string | identifier | "(" , expression , ")" ;

(* Array literal — only valid in field initializer position *)
array_literal   = "[" , array_content , "]" ;
array_content   = repeat_form | list_form ;
repeat_form     = array_elem , ";" , ( dec_number | "_" ) ;
list_form       = array_elem , { "," , array_elem } ;
array_elem      = env_var | hex_number | bin_number | dec_number ;

(* Literals *)
hex_number      = "0x" , hex_digit , { hex_digit } ;
bin_number      = "0b" , ( "0" | "1" ) , { "0" | "1" } ;
dec_number      = digit , { digit } ;
string          = '"' , { string_char } , '"' ;

(* Environment variables *)
env_var         = "${" , identifier , "}" ;

(* Built-in functions *)
builtin_call    = "@" , builtin_name , "(" , [ arg_list ] , ")" ;
builtin_name    = "bytes" | "sizeof" | "offsetof" | "crc32" | "crc" | "sha256" ;
arg_list        = argument , { "," , argument } ;
argument        = range_expr | expression ;     (* range_expr takes priority *)

(* Range expressions — @self with optional slice spec *)
range_expr      = "@self" , [ "[" , range_spec , "]" ] ;
range_spec      = [ range_start ] , ".." , [ range_end ] ;
range_start     = identifier | hex_number | bin_number | dec_number ;
range_end       = identifier ;

(* Identifiers *)
identifier      = ( letter | "_" ) , { letter | digit | "_" } ;
letter          = "a" .. "z" | "A" .. "Z" ;
digit           = "0" .. "9" ;
hex_digit       = digit | "a" .. "f" | "A" .. "F" ;
```

## Comments

Single-line comments start with `//`:

```text
// This is a comment
magic: [u8; 4] = @bytes("FPK");  // Magic number
```

## Reserved Keywords

The following are reserved and cannot be used as identifiers:

- `struct`
- Type names: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`
- Directives: `endian`
- Attributes: `packed`, `align`
- Built-in names: `bytes`, `sizeof`, `offsetof`, `crc32`, `crc`, `sha256`
- Special: `@self`

## Type Safety

### Hard Errors

| Scenario | Error |
|----------|-------|
| `[u8; N] = "string"` (without `@bytes`) | E03001 — use `@bytes("...")` instead |
| `[u16; N] = @bytes(...)` | E03001 — `@bytes` only valid for `[u8; N]` arrays |
| `@crc("unknown-algo", ...)` | E04003 — unknown algorithm name |
| Reference to undefined `${VAR}` | E02001 |

### Warnings

| Scenario | Warning |
|----------|---------|
| Integer value has bits above field width (e.g., `u8 = 0x1FF`) | W03002 ValueTruncated |
| String longer than target array | W03001 StringTruncated |
| Shift amount ≥ 64 (result is always 0) | W04001 ShiftOverflow |

## Implementation Notes

### Default Values

| Type | Default Value |
|------|---------------|
| Scalar types | `0x00` |
| Array types | All elements `0x00` |

### Current Limitations

1. **Single struct per file** — multiple structs are not yet supported
2. **CRC algorithms** — only `crc32` and `crc16-modbus`; more planned
3. **Multiple-section hash** — `@sha256(section_a, section_b)` not yet implemented

## Examples

See [examples/basic.rs](examples/basic.rs) for a complete working example.

---

