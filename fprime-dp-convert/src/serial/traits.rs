//! Serialization traits matching FPrime's Fw::Serializable interface

use crate::error::Result;
use super::{Endianness, SerialBuffer};

/// Trait for types that can be deserialized from FPrime binary format
pub trait Deserialize: Sized {
    /// Deserialize from a buffer with the given endianness
    fn deserialize(buf: &mut SerialBuffer, endian: Endianness) -> Result<Self>;

    /// Deserialize from a buffer using FPrime's default big-endian format
    fn deserialize_be(buf: &mut SerialBuffer) -> Result<Self> {
        Self::deserialize(buf, Endianness::Big)
    }

    /// Get the serialized size of this type in bytes
    ///
    /// For fixed-size types, this is constant. For variable-size types
    /// (strings, arrays), this may depend on the actual data.
    fn serialized_size() -> usize;
}

/// Trait for types that can be serialized to FPrime binary format
pub trait Serialize {
    /// Serialize to a byte vector with the given endianness
    fn serialize(&self, endian: Endianness) -> Vec<u8>;

    /// Serialize using FPrime's default big-endian format
    fn serialize_be(&self) -> Vec<u8> {
        self.serialize(Endianness::Big)
    }
}

// ============================================================================
// Implementations for primitive types
// ============================================================================

impl Deserialize for u8 {
    fn deserialize(buf: &mut SerialBuffer, _endian: Endianness) -> Result<Self> {
        buf.read_u8()
    }

    fn serialized_size() -> usize {
        1
    }
}

impl Serialize for u8 {
    fn serialize(&self, _endian: Endianness) -> Vec<u8> {
        vec![*self]
    }
}

impl Deserialize for i8 {
    fn deserialize(buf: &mut SerialBuffer, _endian: Endianness) -> Result<Self> {
        buf.read_i8()
    }

    fn serialized_size() -> usize {
        1
    }
}

impl Serialize for i8 {
    fn serialize(&self, _endian: Endianness) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl Deserialize for bool {
    fn deserialize(buf: &mut SerialBuffer, _endian: Endianness) -> Result<Self> {
        buf.read_bool()
    }

    fn serialized_size() -> usize {
        1
    }
}

impl Serialize for bool {
    fn serialize(&self, _endian: Endianness) -> Vec<u8> {
        vec![if *self { 1 } else { 0 }]
    }
}

macro_rules! impl_primitive {
    ($ty:ty, $read_fn:ident, $size:expr) => {
        impl Deserialize for $ty {
            fn deserialize(buf: &mut SerialBuffer, endian: Endianness) -> Result<Self> {
                buf.$read_fn(endian)
            }

            fn serialized_size() -> usize {
                $size
            }
        }

        impl Serialize for $ty {
            fn serialize(&self, endian: Endianness) -> Vec<u8> {
                match endian {
                    Endianness::Big => self.to_be_bytes().to_vec(),
                    Endianness::Little => self.to_le_bytes().to_vec(),
                }
            }
        }
    };
}

impl_primitive!(u16, read_u16, 2);
impl_primitive!(i16, read_i16, 2);
impl_primitive!(u32, read_u32, 4);
impl_primitive!(i32, read_i32, 4);
impl_primitive!(u64, read_u64, 8);
impl_primitive!(i64, read_i64, 8);
impl_primitive!(f32, read_f32, 4);
impl_primitive!(f64, read_f64, 8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_roundtrip() {
        let original: u32 = 0x12345678;
        let serialized = original.serialize(Endianness::Big);
        let mut buf = SerialBuffer::new(&serialized);
        let deserialized = u32::deserialize(&mut buf, Endianness::Big).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_f64_roundtrip() {
        let original: f64 = std::f64::consts::PI;
        let serialized = original.serialize(Endianness::Big);
        let mut buf = SerialBuffer::new(&serialized);
        let deserialized = f64::deserialize(&mut buf, Endianness::Big).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_bool_serialization() {
        assert_eq!(true.serialize(Endianness::Big), vec![1]);
        assert_eq!(false.serialize(Endianness::Big), vec![0]);
    }
}
