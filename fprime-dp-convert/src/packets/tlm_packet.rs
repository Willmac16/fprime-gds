//! Telemetry packet parsing
//!
//! Reference: Fw/Tlm/TlmPacket.hpp

use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::serial::{Endianness, SerialBuffer};
use crate::types::time::TimeType;

/// A single telemetry entry within a packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlmEntry {
    /// Channel ID
    pub channel_id: u32,

    /// Timestamp of this telemetry sample
    pub time: TimeType,

    /// Raw serialized value data
    pub raw_data: Vec<u8>,
}

/// Telemetry packet containing one or more channel values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlmPacket {
    /// Packet type descriptor (0x0001 for telemetry)
    pub packet_type: u16,

    /// Telemetry entries
    pub entries: Vec<TlmEntry>,
}

impl TlmPacket {
    /// Expected packet descriptor for telemetry packets
    pub const PACKET_DESCRIPTOR: u16 = 0x0001;

    /// Parse a telemetry packet
    ///
    /// Note: This is a basic implementation. Full parsing requires
    /// dictionary information to know channel value sizes.
    pub fn parse_single_entry(
        data: &[u8],
        value_size: usize,
        endian: Endianness,
    ) -> Result<Self> {
        let mut buf = SerialBuffer::new(data);

        let packet_type = buf.read_u16(endian)?;
        let channel_id = buf.read_u32(endian)?;
        let time = TimeType::deserialize(&mut buf, endian)?;
        let raw_data = buf.read_bytes(value_size)?.to_vec();

        Ok(Self {
            packet_type,
            entries: vec![TlmEntry {
                channel_id,
                time,
                raw_data,
            }],
        })
    }

    // TODO: Add parse method that uses dictionary to determine value sizes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_descriptor() {
        assert_eq!(TlmPacket::PACKET_DESCRIPTOR, 0x0001);
    }
}
