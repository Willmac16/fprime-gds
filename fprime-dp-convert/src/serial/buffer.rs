//! Serial buffer implementation matching FPrime's Fw::SerialBuffer
//!
//! This provides a cursor-based buffer for reading serialized FPrime data.

use crate::error::{Error, Result};
use super::Endianness;

/// A buffer for deserializing FPrime binary data
///
/// Tracks a read cursor position and provides methods for reading
/// primitive types with configurable endianness.
///
/// # Example
///
/// ```
/// use fprime_tlm::serial::{SerialBuffer, Endianness};
///
/// let data = [0x00, 0x01, 0x00, 0x02];
/// let mut buf = SerialBuffer::new(&data);
///
/// assert_eq!(buf.read_u16(Endianness::Big).unwrap(), 1);
/// assert_eq!(buf.read_u16(Endianness::Big).unwrap(), 2);
/// ```
#[derive(Debug)]
pub struct SerialBuffer<'a> {
    /// Underlying data
    data: &'a [u8],
    /// Current read position
    offset: usize,
}

impl<'a> SerialBuffer<'a> {
    /// Create a new buffer wrapping the given data
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Get the current read offset
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the total length of the buffer
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the number of bytes remaining to read
    #[inline]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    /// Check if there are at least `n` bytes remaining
    #[inline]
    pub fn has_remaining(&self, n: usize) -> bool {
        self.remaining() >= n
    }

    /// Reset the read cursor to the beginning
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Set the read cursor to a specific offset
    pub fn seek(&mut self, offset: usize) -> Result<()> {
        if offset > self.data.len() {
            return Err(Error::underflow(0, offset, self.data.len()));
        }
        self.offset = offset;
        Ok(())
    }

    /// Skip forward by `n` bytes
    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.ensure_remaining(n)?;
        self.offset += n;
        Ok(())
    }

    /// Get a slice of the remaining data without advancing the cursor
    pub fn peek_remaining(&self) -> &'a [u8] {
        &self.data[self.offset..]
    }

    /// Get a slice of `n` bytes without advancing the cursor
    pub fn peek(&self, n: usize) -> Result<&'a [u8]> {
        self.ensure_remaining(n)?;
        Ok(&self.data[self.offset..self.offset + n])
    }

    /// Read `n` bytes and advance the cursor
    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        self.ensure_remaining(n)?;
        let slice = &self.data[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }

    /// Read bytes into a fixed-size array
    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.ensure_remaining(N)?;
        let mut arr = [0u8; N];
        arr.copy_from_slice(&self.data[self.offset..self.offset + N]);
        self.offset += N;
        Ok(arr)
    }

    // ========================================================================
    // Primitive type readers
    // ========================================================================

    /// Read a u8
    #[inline]
    pub fn read_u8(&mut self) -> Result<u8> {
        self.ensure_remaining(1)?;
        let val = self.data[self.offset];
        self.offset += 1;
        Ok(val)
    }

    /// Read an i8
    #[inline]
    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    /// Read a bool (0 = false, non-zero = true)
    #[inline]
    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    /// Read a u16 with specified endianness
    #[inline]
    pub fn read_u16(&mut self, endian: Endianness) -> Result<u16> {
        let bytes = self.read_array::<2>()?;
        Ok(match endian {
            Endianness::Big => u16::from_be_bytes(bytes),
            Endianness::Little => u16::from_le_bytes(bytes),
        })
    }

    /// Read an i16 with specified endianness
    #[inline]
    pub fn read_i16(&mut self, endian: Endianness) -> Result<i16> {
        let bytes = self.read_array::<2>()?;
        Ok(match endian {
            Endianness::Big => i16::from_be_bytes(bytes),
            Endianness::Little => i16::from_le_bytes(bytes),
        })
    }

    /// Read a u32 with specified endianness
    #[inline]
    pub fn read_u32(&mut self, endian: Endianness) -> Result<u32> {
        let bytes = self.read_array::<4>()?;
        Ok(match endian {
            Endianness::Big => u32::from_be_bytes(bytes),
            Endianness::Little => u32::from_le_bytes(bytes),
        })
    }

    /// Read an i32 with specified endianness
    #[inline]
    pub fn read_i32(&mut self, endian: Endianness) -> Result<i32> {
        let bytes = self.read_array::<4>()?;
        Ok(match endian {
            Endianness::Big => i32::from_be_bytes(bytes),
            Endianness::Little => i32::from_le_bytes(bytes),
        })
    }

    /// Read a u64 with specified endianness
    #[inline]
    pub fn read_u64(&mut self, endian: Endianness) -> Result<u64> {
        let bytes = self.read_array::<8>()?;
        Ok(match endian {
            Endianness::Big => u64::from_be_bytes(bytes),
            Endianness::Little => u64::from_le_bytes(bytes),
        })
    }

    /// Read an i64 with specified endianness
    #[inline]
    pub fn read_i64(&mut self, endian: Endianness) -> Result<i64> {
        let bytes = self.read_array::<8>()?;
        Ok(match endian {
            Endianness::Big => i64::from_be_bytes(bytes),
            Endianness::Little => i64::from_le_bytes(bytes),
        })
    }

    /// Read an f32 with specified endianness
    ///
    /// Note: FPrime serializes floats by treating their bit pattern as an integer
    #[inline]
    pub fn read_f32(&mut self, endian: Endianness) -> Result<f32> {
        let bytes = self.read_array::<4>()?;
        Ok(match endian {
            Endianness::Big => f32::from_be_bytes(bytes),
            Endianness::Little => f32::from_le_bytes(bytes),
        })
    }

    /// Read an f64 with specified endianness
    #[inline]
    pub fn read_f64(&mut self, endian: Endianness) -> Result<f64> {
        let bytes = self.read_array::<8>()?;
        Ok(match endian {
            Endianness::Big => f64::from_be_bytes(bytes),
            Endianness::Little => f64::from_le_bytes(bytes),
        })
    }

    // ========================================================================
    // FPrime-specific readers (big-endian default)
    // ========================================================================

    /// Read a u16 in FPrime's default big-endian format
    #[inline]
    pub fn read_u16_be(&mut self) -> Result<u16> {
        self.read_u16(Endianness::Big)
    }

    /// Read a u32 in FPrime's default big-endian format
    #[inline]
    pub fn read_u32_be(&mut self) -> Result<u32> {
        self.read_u32(Endianness::Big)
    }

    /// Read a u64 in FPrime's default big-endian format
    #[inline]
    pub fn read_u64_be(&mut self) -> Result<u64> {
        self.read_u64(Endianness::Big)
    }

    /// Read a length-prefixed string (U16 length prefix)
    pub fn read_string(&mut self, endian: Endianness) -> Result<String> {
        let len = self.read_u16(endian)? as usize;
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|e| Error::Dictionary(format!("Invalid UTF-8 string: {}", e)))
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Ensure at least `n` bytes are remaining
    #[inline]
    fn ensure_remaining(&self, n: usize) -> Result<()> {
        if self.remaining() < n {
            Err(Error::underflow(n, self.offset, self.remaining()))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x42, 0xFF];
        let mut buf = SerialBuffer::new(&data);
        assert_eq!(buf.read_u8().unwrap(), 0x42);
        assert_eq!(buf.read_u8().unwrap(), 0xFF);
        assert!(buf.read_u8().is_err());
    }

    #[test]
    fn test_read_u16_big_endian() {
        let data = [0x01, 0x02]; // 258 in big-endian
        let mut buf = SerialBuffer::new(&data);
        assert_eq!(buf.read_u16(Endianness::Big).unwrap(), 0x0102);
    }

    #[test]
    fn test_read_u16_little_endian() {
        let data = [0x01, 0x02]; // 513 in little-endian
        let mut buf = SerialBuffer::new(&data);
        assert_eq!(buf.read_u16(Endianness::Little).unwrap(), 0x0201);
    }

    #[test]
    fn test_read_u32_big_endian() {
        let data = [0x00, 0x00, 0x01, 0x00]; // 256 in big-endian
        let mut buf = SerialBuffer::new(&data);
        assert_eq!(buf.read_u32(Endianness::Big).unwrap(), 256);
    }

    #[test]
    fn test_read_i32_negative() {
        let data = [0xFF, 0xFF, 0xFF, 0xFE]; // -2 in big-endian
        let mut buf = SerialBuffer::new(&data);
        assert_eq!(buf.read_i32(Endianness::Big).unwrap(), -2);
    }

    #[test]
    fn test_read_f32() {
        let data = [0x40, 0x48, 0xF5, 0xC3]; // 3.14 in big-endian IEEE 754
        let mut buf = SerialBuffer::new(&data);
        let val = buf.read_f32(Endianness::Big).unwrap();
        assert!((val - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_offset_tracking() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut buf = SerialBuffer::new(&data);
        assert_eq!(buf.offset(), 0);
        buf.read_u8().unwrap();
        assert_eq!(buf.offset(), 1);
        buf.read_u16(Endianness::Big).unwrap();
        assert_eq!(buf.offset(), 3);
    }

    #[test]
    fn test_remaining() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut buf = SerialBuffer::new(&data);
        assert_eq!(buf.remaining(), 4);
        buf.read_u16(Endianness::Big).unwrap();
        assert_eq!(buf.remaining(), 2);
    }

    #[test]
    fn test_seek() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut buf = SerialBuffer::new(&data);
        buf.seek(2).unwrap();
        assert_eq!(buf.read_u8().unwrap(), 0x03);
    }

    #[test]
    fn test_skip() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut buf = SerialBuffer::new(&data);
        buf.skip(2).unwrap();
        assert_eq!(buf.read_u8().unwrap(), 0x03);
    }

    #[test]
    fn test_buffer_underflow() {
        let data = [0x01];
        let mut buf = SerialBuffer::new(&data);
        let result = buf.read_u32(Endianness::Big);
        assert!(matches!(result, Err(Error::BufferUnderflow { .. })));
    }
}
