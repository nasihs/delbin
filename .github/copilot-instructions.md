# Delbin – Copilot Instructions

## Project Overview

Delbin is a Rust library implementing a DSL ("Descriptive Language for Binary Object") for generating binary data structures — primarily firmware headers — from a human-readable description. The primary public API is `generate()` / `generate_hex()` / `merge()` in `src/lib.rs`.

## Build, Test, and Lint

```bash
# Build
cargo build

# Run all tests (including doc-tests in lib.rs)
cargo test

# Run a single test by name
cargo test test_generate_with_crc32

# Run tests with output visible
cargo test -- --nocapture

# Run examples
cargo run --example basic
cargo run --example array_syntax

# Lint
cargo clippy
```

## Architecture

The pipeline is: **DSL text → Parser → AST → Evaluator → `Vec<u8>`**

| Module | Role |
|---|---|
| `src/parser.rs` + `src/grammar.pest` | PEG parser (pest 2.x) producing a parse tree |
| `src/ast.rs` | AST node types (`File`, `StructDef`, `FieldDef`, `Expr`, …) |
| `src/eval.rs` | `Evaluator` — walks the AST, maintains output buffer and field offsets, handles two-phase evaluation for self-referencing fields |
| `src/builtin.rs` | Implementations of `@bytes`, `@sizeof`, `@offsetof`, `@crc32`, `@sha256` |
| `src/types.rs` | Core value types: `Value` (U64/String/Bytes), `ScalarType`, `Endian` |
| `src/error.rs` | `DelbinError` (with `ErrorCode`), `DelbinWarning` (with `WarningCode`), `Result<T>` alias |
| `src/utils.rs` | Helper functions re-exported from `lib.rs`: `create_env`, `hex_dump`, `to_hex_string`, etc. |

## Key Conventions

### Error Handling
- All errors use `DelbinError` with structured `ErrorCode` (E01xxx parse, E02xxx semantic, E03xxx type, E04xxx evaluation, E05xxx IO).
- Warnings use `DelbinWarning` with `WarningCode` (W03001 string truncated, W03002 value truncated).
- Errors are built with a builder pattern: `DelbinError::new(code, msg).with_location(...).with_hint(...)`.
- `Result<T>` is the crate-local alias `std::result::Result<T, DelbinError>`.
- Use `thiserror` for the `#[derive(Error)]` impl on `DelbinError`.

### Public API Pattern
- The three top-level functions (`generate`, `generate_hex`, `merge`) are the only intended entry points.
- Callers pass `&HashMap<String, Value>` for environment variables and `&HashMap<String, Vec<u8>>` for named binary sections.
- `GenerateResult` carries both `.data: Vec<u8>` and `.warnings: Vec<DelbinWarning>`.
- Utility constructors (`create_env`, `create_sections`, `env_insert_int`, `env_insert_str`) are provided and re-exported from `lib.rs` for ergonomic use.

### Two-Phase Evaluation
Fields that self-reference (e.g., `header_crc: u32 = @crc32(@self[..header_crc])`) are deferred as `PendingField` entries during the first pass (filled with zeros), then resolved in a second pass once the output buffer is fully written. This is the key complexity in `eval.rs`.

### DSL Conventions
- A file contains optional `@endian = little|big;` directive (default: little), then exactly one `struct` definition.
- Only `@packed` struct attribute is currently implemented; `@align(n)` is planned but unimplemented.
- Built-in functions are prefixed with `@`: `@bytes`, `@sizeof`, `@offsetof`, `@crc32`, `@sha256`.
- Environment variables use `${VAR_NAME}` syntax.
- Array length expressions support computed values (e.g., `[u8; 64 - @offsetof(_pad)]`).
- CRC32 uses ISO-HDLC (poly `0x04C11DB7`, init/xorout `0xFFFFFFFF`, reflect in/out).
- `@sizeof(@self)` resolves to the total struct size; `@offsetof(field)` resolves to a field's byte offset (self-referencing `@offsetof` returns the current offset).

### Testing
- Integration tests live in `src/lib.rs` under `#[cfg(test)]`.
- Module-level unit tests are in their respective `src/*.rs` files.
- The `hex` crate (dev-dependency) is available for hex encoding in tests.
- Doc-tests in `lib.rs` are executable and serve as usage examples.

### Known Limitations
- **Single struct per file**: the parser only accepts exactly one `struct` definition.
- `@align(n)` attribute is parsed but not implemented in the evaluator.
- `@self[field..]` range syntax (from-field-to-end) is parsed but not evaluated.
- No `parse()` or `validate()` public APIs yet — only `generate()` / `generate_hex()` / `merge()`.
- Multi-section arguments to `@sha256()` / `@crc32()` are not yet implemented.

### Pitfalls to Avoid
- **Adding a new self-referencing builtin**: `is_self_referencing()` in `eval.rs` hardcodes `@crc32` and `@sha256` by name. Any new builtin that reads from `@self` must be added there or it won't be deferred correctly.
- **`field_offsets` side effects**: `eval.rs` calls `field_offsets.clear()` during the size pre-scan pass. Code that reads `field_offsets` must not assume it is populated until after the main evaluation pass.
- **Array literal placement**: grammar allows `array_literal` in arbitrary expression positions, but it is only semantically valid as a direct field initializer — do not use it inside sub-expressions.
