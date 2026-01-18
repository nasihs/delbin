//! Delbin utility functions

use crate::types::Value;
use std::collections::HashMap;

/// Create environment variable mapping from common types
pub fn create_env() -> HashMap<String, Value> {
    HashMap::new()
}

/// Add integer value to environment variables
pub fn env_insert_int(env: &mut HashMap<String, Value>, key: &str, value: u64) {
    env.insert(key.to_string(), Value::U64(value));
}

/// Add string value to environment variables
pub fn env_insert_str(env: &mut HashMap<String, Value>, key: &str, value: &str) {
    env.insert(key.to_string(), Value::String(value.to_string()));
}

/// Create sections mapping
pub fn create_sections() -> HashMap<String, Vec<u8>> {
    HashMap::new()
}

/// Format byte array as hexadecimal string
pub fn to_hex_string(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join("")
}

/// Parse hexadecimal string to byte array
pub fn from_hex_string(hex: &str) -> Option<Vec<u8>> {
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return None;
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

/// Print byte array as formatted hexadecimal dump
pub fn hex_dump(data: &[u8], bytes_per_line: usize) -> String {
    let mut result = String::new();

    for (i, chunk) in data.chunks(bytes_per_line).enumerate() {
        // Address
        result.push_str(&format!("{:08X}: ", i * bytes_per_line));

        // Hexadecimal
        for byte in chunk {
            result.push_str(&format!("{:02X} ", byte));
        }

        // Padding
        for _ in 0..(bytes_per_line - chunk.len()) {
            result.push_str("   ");
        }

        // ASCII
        result.push_str(" |");
        for byte in chunk {
            let c = if *byte >= 0x20 && *byte < 0x7F {
                *byte as char
            } else {
                '.'
            };
            result.push(c);
        }
        result.push_str("|\n");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex_string() {
        assert_eq!(to_hex_string(&[0xDE, 0xAD, 0xBE, 0xEF]), "DEADBEEF");
    }

    #[test]
    fn test_from_hex_string() {
        assert_eq!(
            from_hex_string("DEADBEEF"),
            Some(vec![0xDE, 0xAD, 0xBE, 0xEF])
        );
        assert_eq!(from_hex_string("deadbeef"), Some(vec![0xDE, 0xAD, 0xBE, 0xEF]));
        assert_eq!(from_hex_string("123"), None); // Odd length
    }

    #[test]
    fn test_hex_dump() {
        let data = b"Hello, World!";
        let dump = hex_dump(data, 16);
        assert!(dump.contains("48 65 6C 6C"));
    }
}
