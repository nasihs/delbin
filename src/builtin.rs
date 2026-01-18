//! Delbin built-in function implementations

use crc::{Crc, CRC_32_ISO_HDLC};
use sha2::{Digest, Sha256};

use crate::error::{WarningCode, DelbinWarning};

/// CRC32 calculation (ISO-HDLC)
pub fn crc32(data: &[u8]) -> u32 {
    const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    CRC.checksum(data)
}

/// SHA256 calculation
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// @bytes() function: convert string to byte array
pub fn bytes(s: &str, target_len: usize) -> (Vec<u8>, Option<DelbinWarning>) {
    let bytes = s.as_bytes();
    let mut result = vec![0u8; target_len];
    let mut warning = None;

    if bytes.len() > target_len {
        // Truncate and warn
        result.copy_from_slice(&bytes[..target_len]);
        warning = Some(DelbinWarning {
            code: WarningCode::W03001,
            message: format!(
                "String '{}' truncated from {} to {} bytes",
                s,
                bytes.len(),
                target_len
            ),
            location: None,
        });
    } else {
        // Copy and zero-fill
        result[..bytes.len()].copy_from_slice(bytes);
    }

    (result, warning)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        let data = b"hello world";
        let crc = crc32(data);
        assert_eq!(crc, 0x0D4A1185);
    }

    #[test]
    fn test_sha256() {
        let data = b"hello world";
        let hash = sha256(data);
        assert_eq!(
            hex::encode(hash),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_bytes() {
        let (result, warning) = bytes("fpk", 4);
        assert_eq!(result, vec![0x66, 0x70, 0x6B, 0x00]);
        assert!(warning.is_none());

        let (result, warning) = bytes("hello", 3);
        assert_eq!(result, vec![0x68, 0x65, 0x6C]);
        assert!(warning.is_some());
    }
}
