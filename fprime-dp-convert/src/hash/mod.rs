//! Hash/checksum utilities for data validation
//!
//! FPrime supports both CRC32 and SHA256 for data integrity.

/// Compute CRC32 checksum of data
pub fn crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

/// Compute CRC32 checksum and return as big-endian bytes
pub fn crc32_bytes(data: &[u8]) -> [u8; 4] {
    crc32fast::hash(data).to_be_bytes()
}

/// Compute SHA256 hash of data
pub fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    Sha256::digest(data).into()
}

/// Verify CRC32 checksum
pub fn verify_crc32(data: &[u8], expected: &[u8; 4]) -> bool {
    let computed = crc32_bytes(data);
    &computed == expected
}

/// Verify SHA256 hash
pub fn verify_sha256(data: &[u8], expected: &[u8; 32]) -> bool {
    let computed = sha256(data);
    &computed == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_known_value() {
        // CRC32 of "123456789" is 0xCBF43926
        let data = b"123456789";
        assert_eq!(crc32(data), 0xCBF43926);
    }

    #[test]
    fn test_sha256_known_value() {
        // SHA256 of empty string
        let data = b"";
        let hash = sha256(data);
        // First byte of SHA256("") is 0xe3
        assert_eq!(hash[0], 0xe3);
    }

    #[test]
    fn test_verify_crc32() {
        let data = b"test data";
        let hash = crc32_bytes(data);
        assert!(verify_crc32(data, &hash));
        assert!(!verify_crc32(b"wrong data", &hash));
    }
}
