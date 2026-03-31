//! Error types for FPrime telemetry parsing

use thiserror::Error;

/// Result type alias for this crate
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during FPrime telemetry parsing
#[derive(Debug, Error)]
pub enum Error {
    /// Buffer does not contain enough bytes for the requested operation
    #[error("Buffer underflow: needed {needed} bytes at offset {offset}, but only {available} available")]
    BufferUnderflow {
        needed: usize,
        offset: usize,
        available: usize,
    },

    /// Packet descriptor does not match expected value
    #[error("Invalid packet descriptor: expected {expected:#06x}, got {got:#06x}")]
    InvalidDescriptor { expected: u16, got: u16 },

    /// Hash validation failed
    #[error("Hash validation failed for {location}: expected {expected}, computed {computed}")]
    HashMismatch {
        location: &'static str,
        expected: String,
        computed: String,
    },

    /// Unknown type encountered during deserialization
    #[error("Unknown type: '{0}'")]
    UnknownType(String),

    /// Type resolution failed (e.g., circular alias)
    #[error("Failed to resolve type '{name}': {reason}")]
    TypeResolution { name: String, reason: String },

    /// Invalid enum value
    #[error("Invalid enum value {value} for type '{type_name}'")]
    InvalidEnumValue { type_name: String, value: i64 },

    /// Data product state is invalid
    #[error("Invalid data product state: {0}")]
    InvalidDpState(u8),

    /// Dictionary parsing error
    #[error("Dictionary error: {0}")]
    Dictionary(String),

    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// HDF5 error (when feature enabled)
    #[cfg(feature = "hdf5")]
    #[error("HDF5 error: {0}")]
    Hdf5(#[from] hdf5::Error),
}

impl Error {
    /// Create a buffer underflow error
    pub fn underflow(needed: usize, offset: usize, available: usize) -> Self {
        Self::BufferUnderflow {
            needed,
            offset,
            available,
        }
    }

    /// Create a hash mismatch error
    pub fn hash_mismatch(location: &'static str, expected: &[u8], computed: &[u8]) -> Self {
        Self::HashMismatch {
            location,
            expected: hex::encode(expected),
            computed: hex::encode(computed),
        }
    }
}

/// Helper module for hex encoding (inline to avoid extra dependency)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
