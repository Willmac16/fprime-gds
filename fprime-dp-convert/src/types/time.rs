//! FPrime TimeType implementation
//!
//! Reference: Fw/Time/Time.fpp, Fw/Time/Time.hpp
//!
//! The TimeType is a 12-byte structure (based on FSW exploration):
//! - Time Base (U16): 2 bytes
//! - Time Context (U8): 1 byte
//! - Seconds (U32): 4 bytes
//! - Microseconds (U32): 4 bytes
//! Total: 11 bytes
//!
//! Note: FSW documentation says 12 bytes but the actual layout is 11.
//! This should be verified against actual FSW behavior.

use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::serial::{Endianness, SerialBuffer};
use crate::serial::traits::Deserialize as DeserializeTrait;

/// FPrime Time Base enumeration
///
/// Indicates the reference clock for the timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum TimeBase {
    /// No time base established
    None = 0,
    /// Processor cycle time
    ProcTime = 1,
    /// Workstation time (for testing)
    WorkstationTime = 2,
    /// Spacecraft clock time
    SpacecraftTime = 3,
    /// Placeholder for sequences (don't care about actual time)
    DontCare = 0xFFFF,
}

impl TimeBase {
    /// Convert from a u16 value
    pub fn from_u16(value: u16) -> Self {
        match value {
            0 => Self::None,
            1 => Self::ProcTime,
            2 => Self::WorkstationTime,
            3 => Self::SpacecraftTime,
            0xFFFF => Self::DontCare,
            // Unknown values default to None
            _ => Self::None,
        }
    }

    /// Convert to a u16 value
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

impl Default for TimeBase {
    fn default() -> Self {
        Self::None
    }
}

/// FPrime Time Type
///
/// Represents a timestamp with base, context, seconds, and microseconds.
///
/// # Serialized Format (11 bytes, Big-Endian)
///
/// | Offset | Size | Field         | Type |
/// |--------|------|---------------|------|
/// | 0      | 2    | time_base     | U16  |
/// | 2      | 1    | time_context  | U8   |
/// | 3      | 4    | seconds       | U32  |
/// | 7      | 4    | useconds      | U32  |
///
/// # Example
///
/// ```
/// use fprime_tlm::types::time::{TimeType, TimeBase};
/// use fprime_tlm::serial::{SerialBuffer, Endianness};
///
/// let time = TimeType {
///     time_base: TimeBase::SpacecraftTime,
///     time_context: 0,
///     seconds: 1234567890,
///     useconds: 123456,
/// };
///
/// // Convert to Unix timestamp
/// let unix_ts = time.to_unix_timestamp();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TimeType {
    /// Time base indicating the reference clock
    pub time_base: TimeBase,
    /// Application-defined context (0-255)
    pub time_context: u8,
    /// Seconds since epoch/base
    pub seconds: u32,
    /// Microseconds (0-999999)
    pub useconds: u32,
}

impl TimeType {
    /// Serialized size in bytes
    /// Note: FSW says 12 bytes but actual layout appears to be 11
    /// Using 11 based on field sizes: 2 + 1 + 4 + 4 = 11
    /// If validation fails, this may need to be 12 (with padding)
    pub const SERIALIZED_SIZE: usize = 11;

    /// Alternative size if there's padding (FSW docs say 12)
    pub const SERIALIZED_SIZE_PADDED: usize = 12;

    /// Create a new TimeType with the given values
    pub fn new(time_base: TimeBase, time_context: u8, seconds: u32, useconds: u32) -> Self {
        Self {
            time_base,
            time_context,
            seconds,
            useconds,
        }
    }

    /// Create a TimeType from Unix timestamp (seconds.microseconds)
    pub fn from_unix(unix_secs: f64, time_base: TimeBase, time_context: u8) -> Self {
        let seconds = unix_secs.trunc() as u32;
        let useconds = ((unix_secs.fract()) * 1_000_000.0) as u32;
        Self {
            time_base,
            time_context,
            seconds,
            useconds,
        }
    }

    /// Convert to Unix timestamp as f64 (seconds.microseconds)
    pub fn to_unix_timestamp(&self) -> f64 {
        self.seconds as f64 + (self.useconds as f64 / 1_000_000.0)
    }

    /// Convert to total microseconds since epoch
    pub fn to_total_useconds(&self) -> u64 {
        (self.seconds as u64) * 1_000_000 + (self.useconds as u64)
    }

    /// Deserialize from a buffer
    pub fn deserialize(buf: &mut SerialBuffer, endian: Endianness) -> Result<Self> {
        let time_base = TimeBase::from_u16(buf.read_u16(endian)?);
        let time_context = buf.read_u8()?;
        let seconds = buf.read_u32(endian)?;
        let useconds = buf.read_u32(endian)?;

        Ok(Self {
            time_base,
            time_context,
            seconds,
            useconds,
        })
    }

    /// Deserialize with padding byte (if FSW uses 12 bytes)
    pub fn deserialize_padded(buf: &mut SerialBuffer, endian: Endianness) -> Result<Self> {
        let time_base = TimeBase::from_u16(buf.read_u16(endian)?);
        let time_context = buf.read_u8()?;
        let _padding = buf.read_u8()?; // Skip padding byte
        let seconds = buf.read_u32(endian)?;
        let useconds = buf.read_u32(endian)?;

        Ok(Self {
            time_base,
            time_context,
            seconds,
            useconds,
        })
    }

    /// Serialize to bytes
    pub fn serialize(&self, endian: Endianness) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SERIALIZED_SIZE);

        let base_bytes = match endian {
            Endianness::Big => self.time_base.as_u16().to_be_bytes(),
            Endianness::Little => self.time_base.as_u16().to_le_bytes(),
        };
        bytes.extend_from_slice(&base_bytes);
        bytes.push(self.time_context);

        let sec_bytes = match endian {
            Endianness::Big => self.seconds.to_be_bytes(),
            Endianness::Little => self.seconds.to_le_bytes(),
        };
        bytes.extend_from_slice(&sec_bytes);

        let usec_bytes = match endian {
            Endianness::Big => self.useconds.to_be_bytes(),
            Endianness::Little => self.useconds.to_le_bytes(),
        };
        bytes.extend_from_slice(&usec_bytes);

        bytes
    }

    /// Check if the time is valid (useconds < 1,000,000)
    pub fn is_valid(&self) -> bool {
        self.useconds < 1_000_000
    }
}

impl std::fmt::Display for TimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{:06} (base={:?}, ctx={})",
            self.seconds, self.useconds, self.time_base, self.time_context
        )
    }
}

impl PartialOrd for TimeType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimeType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.seconds.cmp(&other.seconds) {
            std::cmp::Ordering::Equal => self.useconds.cmp(&other.useconds),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_type_size() {
        // Verify the expected size
        assert_eq!(TimeType::SERIALIZED_SIZE, 11);
    }

    #[test]
    fn test_time_base_conversion() {
        assert_eq!(TimeBase::from_u16(0), TimeBase::None);
        assert_eq!(TimeBase::from_u16(1), TimeBase::ProcTime);
        assert_eq!(TimeBase::from_u16(2), TimeBase::WorkstationTime);
        assert_eq!(TimeBase::from_u16(3), TimeBase::SpacecraftTime);
        assert_eq!(TimeBase::from_u16(0xFFFF), TimeBase::DontCare);
        // Unknown values default to None
        assert_eq!(TimeBase::from_u16(99), TimeBase::None);
    }

    #[test]
    fn test_time_type_roundtrip() {
        let original = TimeType {
            time_base: TimeBase::SpacecraftTime,
            time_context: 42,
            seconds: 1234567890,
            useconds: 123456,
        };

        let serialized = original.serialize(Endianness::Big);
        assert_eq!(serialized.len(), TimeType::SERIALIZED_SIZE);

        let mut buf = SerialBuffer::new(&serialized);
        let deserialized = TimeType::deserialize(&mut buf, Endianness::Big).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_time_type_big_endian_format() {
        let time = TimeType {
            time_base: TimeBase::SpacecraftTime, // 0x0003
            time_context: 0x42,
            seconds: 0x12345678,
            useconds: 0x000186A0, // 100000
        };

        let bytes = time.serialize(Endianness::Big);

        // Check byte layout
        assert_eq!(bytes[0], 0x00); // time_base MSB
        assert_eq!(bytes[1], 0x03); // time_base LSB
        assert_eq!(bytes[2], 0x42); // time_context
        assert_eq!(bytes[3], 0x12); // seconds MSB
        assert_eq!(bytes[4], 0x34);
        assert_eq!(bytes[5], 0x56);
        assert_eq!(bytes[6], 0x78); // seconds LSB
        assert_eq!(bytes[7], 0x00); // useconds MSB
        assert_eq!(bytes[8], 0x01);
        assert_eq!(bytes[9], 0x86);
        assert_eq!(bytes[10], 0xA0); // useconds LSB
    }

    #[test]
    fn test_to_unix_timestamp() {
        let time = TimeType {
            time_base: TimeBase::SpacecraftTime,
            time_context: 0,
            seconds: 1000,
            useconds: 500000, // 0.5 seconds
        };

        let unix = time.to_unix_timestamp();
        assert!((unix - 1000.5).abs() < 0.000001);
    }

    #[test]
    fn test_to_total_useconds() {
        let time = TimeType {
            time_base: TimeBase::SpacecraftTime,
            time_context: 0,
            seconds: 1,
            useconds: 500000,
        };

        assert_eq!(time.to_total_useconds(), 1_500_000);
    }

    #[test]
    fn test_time_ordering() {
        let t1 = TimeType::new(TimeBase::None, 0, 100, 0);
        let t2 = TimeType::new(TimeBase::None, 0, 100, 500000);
        let t3 = TimeType::new(TimeBase::None, 0, 101, 0);

        assert!(t1 < t2);
        assert!(t2 < t3);
        assert!(t1 < t3);
    }

    #[test]
    fn test_is_valid() {
        let valid = TimeType::new(TimeBase::None, 0, 0, 999999);
        let invalid = TimeType::new(TimeBase::None, 0, 0, 1000000);

        assert!(valid.is_valid());
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_display() {
        let time = TimeType::new(TimeBase::SpacecraftTime, 1, 12345, 67890);
        let s = format!("{}", time);
        assert!(s.contains("12345.067890"));
        assert!(s.contains("SpacecraftTime"));
    }
}
