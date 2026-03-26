//! Delbin built-in function implementations

use crc::{Crc, CRC_16_IBM_3740, CRC_16_MODBUS, CRC_32_ISO_HDLC, CRC_32_MPEG_2};
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256};

use crate::error::{DelbinError, ErrorCode, Result, WarningCode, DelbinWarning};

/// CRC32 calculation (ISO-HDLC)
pub fn crc32(data: &[u8]) -> u32 {
    const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    CRC.checksum(data)
}

/// CRC16 calculation (CCITT: poly=0x1021, init=0xFFFF, refin=false, refout=false, xorout=0x0000)
pub fn crc16(data: &[u8]) -> u16 {
    const CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_3740);
    CRC.checksum(data)
}

/// Generic CRC calculation
///
/// Supported algorithms: "crc32", "crc32-mpeg2", "crc16", "crc16-modbus"
pub fn crc_generic(algorithm: &str, data: &[u8]) -> Result<u64> {
    match algorithm {
        "crc32" => Ok(crc32(data) as u64),
        "crc32-mpeg2" => {
            const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_MPEG_2);
            Ok(CRC.checksum(data) as u64)
        }
        "crc16" => Ok(crc16(data) as u64),
        "crc16-modbus" => {
            const CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);
            Ok(CRC.checksum(data) as u64)
        }
        _ => Err(DelbinError::new(
            ErrorCode::E04003,
            format!("Unknown CRC algorithm: '{}'", algorithm),
        )),
    }
}

/// SHA256 calculation
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Generic hash calculation
///
/// Supported algorithms: "sha256" (32 bytes), "sha1" (20 bytes), "md5" (16 bytes)
pub fn hash_generic(algorithm: &str, data: &[u8]) -> Result<Vec<u8>> {
    match algorithm {
        "sha256" => Ok(sha256(data).to_vec()),
        "sha1" => {
            let mut hasher = Sha1::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        "md5" => {
            let mut hasher = Md5::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        _ => Err(DelbinError::new(
            ErrorCode::E04003,
            format!("Unknown hash algorithm: '{}'", algorithm),
        )),
    }
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
    fn test_crc16() {
        let data = b"hello world";
        let crc = crc16(data);
        // CRC16-CCITT (IBM-3740: poly=0x1021, init=0xFFFF, refin=false, refout=false)
        assert_eq!(crc, 61419);
    }

    #[test]
    fn test_crc_generic_crc32() {
        let data = b"hello world";
        let crc = crc_generic("crc32", data).unwrap();
        assert_eq!(crc, 0x0D4A1185);
    }

    #[test]
    fn test_crc_generic_crc16() {
        let data = b"hello world";
        let crc = crc_generic("crc16", data).unwrap();
        assert_eq!(crc, 61419);
    }

    #[test]
    fn test_crc_generic_crc16_modbus() {
        let data = b"hello world";
        let result = crc_generic("crc16-modbus", data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_crc_generic_crc32_mpeg2() {
        let data = b"hello world";
        let result = crc_generic("crc32-mpeg2", data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_crc_generic_unknown() {
        let data = b"hello world";
        let result = crc_generic("unknown", data);
        assert!(result.is_err());
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
    fn test_hash_generic_sha256() {
        let data = b"hello world";
        let hash = hash_generic("sha256", data).unwrap();
        assert_eq!(
            hex::encode(&hash),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_generic_sha1() {
        let data = b"hello world";
        let hash = hash_generic("sha1", data).unwrap();
        assert_eq!(hash.len(), 20);
        assert_eq!(hex::encode(&hash), "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }

    #[test]
    fn test_hash_generic_md5() {
        let data = b"hello world";
        let hash = hash_generic("md5", data).unwrap();
        assert_eq!(hash.len(), 16);
        assert_eq!(hex::encode(&hash), "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }

    #[test]
    fn test_hash_generic_unknown() {
        let data = b"hello world";
        let result = hash_generic("unknown", data);
        assert!(result.is_err());
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
