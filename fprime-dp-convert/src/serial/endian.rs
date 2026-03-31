//! Endianness handling for FPrime serialization
//!
//! FPrime defaults to big-endian (network byte order) for all serialization.

/// Byte order for serialization/deserialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Endianness {
    /// Big-endian (network byte order) - FPrime default
    #[default]
    Big,
    /// Little-endian (host byte order on x86)
    Little,
}

impl Endianness {
    /// Check if this is big-endian
    #[inline]
    pub fn is_big(self) -> bool {
        matches!(self, Self::Big)
    }

    /// Check if this is little-endian
    #[inline]
    pub fn is_little(self) -> bool {
        matches!(self, Self::Little)
    }
}
