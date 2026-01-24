//! Data Product Container parsing
//!
//! Reference: Fw/Dp/DpContainer.hpp
//!
//! ## Header Layout (58 bytes)
//!
//! | Offset | Size | Field              | Type                   |
//! |--------|------|--------------------|------------------------|
//! | 0      | 2    | Packet Descriptor  | U16 (always 0x0005)    |
//! | 2      | 4    | Container ID       | U32                    |
//! | 6      | 4    | Priority           | U32                    |
//! | 10     | 11   | Time Tag           | TimeType (11 bytes)    |
//! | 21     | 1    | Processing Types   | U8 (bitmask)           |
//! | 22     | 32   | User Data          | [u8; 32]               |
//! | 54     | 1    | DP State           | U8 (enum)              |
//! | 55     | 2    | Data Size          | U16                    |
//! | 57     | 1    | (padding/reserved) | U8                     |
//!
//! Note: The exact layout should be verified against FSW. The offsets
//! above are based on code exploration but may need adjustment.

use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};
use crate::serial::{Endianness, SerialBuffer};
use crate::types::time::TimeType;
use super::descriptor::PacketDescriptor;

/// Default size of user data field in DP container
pub const CONTAINER_USER_DATA_SIZE: usize = 32;

/// Data Product state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum DpState {
    /// Data product has not been transmitted
    Untransmitted = 0,
    /// Data product is partially filled
    Partial = 1,
    /// Data product has been transmitted
    Transmitted = 2,
}

impl DpState {
    /// Convert from a u8 value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Untransmitted),
            1 => Ok(Self::Partial),
            2 => Ok(Self::Transmitted),
            v => Err(Error::InvalidDpState(v)),
        }
    }
}

/// Processing type flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ProcTypes(u8);

impl ProcTypes {
    pub const PROC_TYPE_ZERO: u8 = 0x01;
    pub const PROC_TYPE_ONE: u8 = 0x02;
    pub const PROC_TYPE_TWO: u8 = 0x04;

    /// Create from raw u8 value
    pub fn from_u8(value: u8) -> Self {
        Self(value)
    }

    /// Get the raw u8 value
    pub fn as_u8(self) -> u8 {
        self.0
    }

    /// Check if a specific processing type is set
    pub fn has(&self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }
}

/// Data Product Container Header
///
/// This is the fixed 58-byte header at the start of every DP container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpContainerHeader {
    /// Packet descriptor (always 0x0005 for DP)
    pub packet_descriptor: u16,

    /// Container ID from dictionary
    pub container_id: u32,

    /// Priority level
    pub priority: u32,

    /// Timestamp when container was created/filled
    pub time_tag: TimeType,

    /// Processing type flags
    pub proc_types: ProcTypes,

    /// User-defined data (32 bytes by default)
    pub user_data: Vec<u8>,

    /// Current state of the data product
    pub dp_state: DpState,

    /// Size of the data payload in bytes
    pub data_size: u16,
}

impl DpContainerHeader {
    /// Size of the header in bytes (based on FSW DpContainer.hpp)
    /// 2 + 4 + 4 + 12 + 1 + 32 + 1 + 2 = 58 bytes
    pub const SIZE: usize = 58;

    /// Field offsets within the header
    pub const PACKET_DESCRIPTOR_OFFSET: usize = 0;
    pub const ID_OFFSET: usize = 2;
    pub const PRIORITY_OFFSET: usize = 6;
    pub const TIME_TAG_OFFSET: usize = 10;
    pub const PROC_TYPES_OFFSET: usize = 22;
    pub const USER_DATA_OFFSET: usize = 23;
    pub const DP_STATE_OFFSET: usize = 55;
    pub const DATA_SIZE_OFFSET: usize = 56;

    /// Parse a header from binary data
    pub fn parse(buf: &mut SerialBuffer, endian: Endianness) -> Result<Self> {
        // Read packet descriptor and validate
        let packet_descriptor = buf.read_u16(endian)?;
        if packet_descriptor != PacketDescriptor::DataProduct.as_u16() {
            return Err(Error::InvalidDescriptor {
                expected: PacketDescriptor::DataProduct.as_u16(),
                got: packet_descriptor,
            });
        }

        // Read remaining header fields
        let container_id = buf.read_u32(endian)?;
        let priority = buf.read_u32(endian)?;

        // TimeType is 12 bytes in FSW (includes padding)
        let time_tag = TimeType::deserialize_padded(buf, endian)?;

        let proc_types = ProcTypes::from_u8(buf.read_u8()?);

        // Read user data
        let user_data = buf.read_bytes(CONTAINER_USER_DATA_SIZE)?.to_vec();

        let dp_state = DpState::from_u8(buf.read_u8()?)?;
        let data_size = buf.read_u16(endian)?;

        Ok(Self {
            packet_descriptor,
            container_id,
            priority,
            time_tag,
            proc_types,
            user_data,
            dp_state,
            data_size,
        })
    }
}

/// Hash digest (can be CRC32 or SHA256)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashDigest {
    /// CRC32 (4 bytes)
    Crc32([u8; 4]),
    /// SHA256 (32 bytes)
    Sha256([u8; 32]),
}

impl HashDigest {
    /// Size of CRC32 digest
    pub const CRC32_SIZE: usize = 4;
    /// Size of SHA256 digest
    pub const SHA256_SIZE: usize = 32;

    /// Get the size of this digest
    pub fn size(&self) -> usize {
        match self {
            Self::Crc32(_) => Self::CRC32_SIZE,
            Self::Sha256(_) => Self::SHA256_SIZE,
        }
    }

    /// Get the bytes of this digest
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Crc32(b) => b,
            Self::Sha256(b) => b,
        }
    }
}

/// A record within a Data Product container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpRecord {
    /// Record ID from dictionary
    pub id: u32,

    /// Record name (from dictionary, if available)
    pub name: Option<String>,

    /// Raw serialized data
    pub raw_data: Vec<u8>,

    // TODO: Add deserialized value once dictionary integration is complete
    // pub value: Option<FprimeValue>,
}

/// Complete Data Product Container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpContainer {
    /// Container header
    pub header: DpContainerHeader,

    /// Header hash (for validation)
    pub header_hash: HashDigest,

    /// Records contained in this data product
    pub records: Vec<DpRecord>,

    /// Data hash (for validation)
    pub data_hash: HashDigest,

    /// Whether the hashes validated successfully
    pub hash_valid: bool,
}

impl DpContainer {
    /// Parse a complete Data Product container from binary data
    ///
    /// # Arguments
    ///
    /// * `data` - Raw binary data of the DP container
    /// * `hash_size` - Size of hash digest (4 for CRC32, 32 for SHA256)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use fprime_tlm::packets::dp_container::{DpContainer, HashDigest};
    ///
    /// let data = std::fs::read("data_product.bin")?;
    /// let container = DpContainer::parse(&data, HashDigest::CRC32_SIZE)?;
    /// println!("Container ID: {}", container.header.container_id);
    /// # Ok::<(), fprime_tlm::Error>(())
    /// ```
    pub fn parse(data: &[u8], hash_size: usize) -> Result<Self> {
        let endian = Endianness::Big;
        let mut buf = SerialBuffer::new(data);

        // Parse header
        let header = DpContainerHeader::parse(&mut buf, endian)?;

        // Read header hash
        let header_hash = Self::read_hash(&mut buf, hash_size)?;

        // Read data payload
        let data_start = buf.offset();
        let data_bytes = buf.read_bytes(header.data_size as usize)?;

        // Read data hash
        let data_hash = Self::read_hash(&mut buf, hash_size)?;

        // Validate hashes
        let header_bytes = &data[0..DpContainerHeader::SIZE];
        let hash_valid = Self::validate_hashes(
            header_bytes,
            data_bytes,
            &header_hash,
            &data_hash,
        );

        // TODO: Parse records from data_bytes using dictionary
        // For now, store raw data as a single record
        let records = vec![DpRecord {
            id: header.container_id,
            name: None,
            raw_data: data_bytes.to_vec(),
        }];

        Ok(Self {
            header,
            header_hash,
            records,
            data_hash,
            hash_valid,
        })
    }

    /// Read a hash digest from the buffer
    fn read_hash(buf: &mut SerialBuffer, size: usize) -> Result<HashDigest> {
        match size {
            HashDigest::CRC32_SIZE => {
                let bytes = buf.read_array::<4>()?;
                Ok(HashDigest::Crc32(bytes))
            }
            HashDigest::SHA256_SIZE => {
                let bytes = buf.read_array::<32>()?;
                Ok(HashDigest::Sha256(bytes))
            }
            _ => Err(Error::Dictionary(format!(
                "Invalid hash size: {}. Expected 4 (CRC32) or 32 (SHA256)",
                size
            ))),
        }
    }

    /// Validate header and data hashes
    fn validate_hashes(
        header_bytes: &[u8],
        data_bytes: &[u8],
        expected_header_hash: &HashDigest,
        expected_data_hash: &HashDigest,
    ) -> bool {
        match (expected_header_hash, expected_data_hash) {
            (HashDigest::Crc32(expected_h), HashDigest::Crc32(expected_d)) => {
                let computed_h = crc32fast::hash(header_bytes);
                let computed_d = crc32fast::hash(data_bytes);

                let expected_h_val = u32::from_be_bytes(*expected_h);
                let expected_d_val = u32::from_be_bytes(*expected_d);

                computed_h == expected_h_val && computed_d == expected_d_val
            }
            (HashDigest::Sha256(expected_h), HashDigest::Sha256(expected_d)) => {
                use sha2::{Sha256, Digest};

                let computed_h: [u8; 32] = Sha256::digest(header_bytes).into();
                let computed_d: [u8; 32] = Sha256::digest(data_bytes).into();

                &computed_h == expected_h && &computed_d == expected_d
            }
            _ => false, // Mismatched hash types
        }
    }

    /// Get the total size of a DP container given the data size and hash size
    pub fn total_size(data_size: usize, hash_size: usize) -> usize {
        DpContainerHeader::SIZE + hash_size + data_size + hash_size
    }

    /// Get the minimum valid container size
    pub fn min_size(hash_size: usize) -> usize {
        Self::total_size(0, hash_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        assert_eq!(DpContainerHeader::SIZE, 58);
    }

    #[test]
    fn test_dp_state_from_u8() {
        assert_eq!(DpState::from_u8(0).unwrap(), DpState::Untransmitted);
        assert_eq!(DpState::from_u8(1).unwrap(), DpState::Partial);
        assert_eq!(DpState::from_u8(2).unwrap(), DpState::Transmitted);
        assert!(DpState::from_u8(3).is_err());
    }

    #[test]
    fn test_proc_types() {
        let pt = ProcTypes::from_u8(0x03);
        assert!(pt.has(ProcTypes::PROC_TYPE_ZERO));
        assert!(pt.has(ProcTypes::PROC_TYPE_ONE));
        assert!(!pt.has(ProcTypes::PROC_TYPE_TWO));
    }

    #[test]
    fn test_hash_digest_size() {
        let crc = HashDigest::Crc32([0; 4]);
        let sha = HashDigest::Sha256([0; 32]);

        assert_eq!(crc.size(), 4);
        assert_eq!(sha.size(), 32);
    }

    #[test]
    fn test_min_container_size() {
        assert_eq!(DpContainer::min_size(4), 58 + 4 + 0 + 4);
        assert_eq!(DpContainer::min_size(32), 58 + 32 + 0 + 32);
    }

    // TODO: Add integration tests with actual DP binary files
}
