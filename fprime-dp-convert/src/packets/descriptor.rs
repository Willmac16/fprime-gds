//! Packet descriptor (APID) definitions
//!
//! Reference: Fw/Com/ComPacket.hpp, default/config/ComCfg.fpp

use serde::{Deserialize, Serialize};

/// FPrime packet type descriptor (APID)
///
/// This is the first 2 bytes of every FPrime packet, identifying its type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum PacketDescriptor {
    /// Command packet
    Command = 0x0000,
    /// Individual telemetry channel
    Telemetry = 0x0001,
    /// Event/log message
    Log = 0x0002,
    /// File transfer packet
    File = 0x0003,
    /// Packetized telemetry (multiple channels)
    PacketizedTelemetry = 0x0004,
    /// Data Product container
    DataProduct = 0x0005,
    /// Idle packet (for padding)
    Idle = 0x0006,
    /// Handshake packet
    Handshake = 0x00FE,
    /// Unknown packet type
    Unknown = 0x00FF,
}

impl PacketDescriptor {
    /// Convert from a u16 value
    pub fn from_u16(value: u16) -> Self {
        match value {
            0x0000 => Self::Command,
            0x0001 => Self::Telemetry,
            0x0002 => Self::Log,
            0x0003 => Self::File,
            0x0004 => Self::PacketizedTelemetry,
            0x0005 => Self::DataProduct,
            0x0006 => Self::Idle,
            0x00FE => Self::Handshake,
            _ => Self::Unknown,
        }
    }

    /// Convert to a u16 value
    pub fn as_u16(self) -> u16 {
        self as u16
    }

    /// Get the human-readable name of this packet type
    pub fn name(self) -> &'static str {
        match self {
            Self::Command => "Command",
            Self::Telemetry => "Telemetry",
            Self::Log => "Log",
            Self::File => "File",
            Self::PacketizedTelemetry => "PacketizedTelemetry",
            Self::DataProduct => "DataProduct",
            Self::Idle => "Idle",
            Self::Handshake => "Handshake",
            Self::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for PacketDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (0x{:04X})", self.name(), self.as_u16())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_descriptor_values() {
        assert_eq!(PacketDescriptor::DataProduct.as_u16(), 0x0005);
        assert_eq!(PacketDescriptor::Telemetry.as_u16(), 0x0001);
    }

    #[test]
    fn test_from_u16() {
        assert_eq!(PacketDescriptor::from_u16(0x0005), PacketDescriptor::DataProduct);
        assert_eq!(PacketDescriptor::from_u16(0x9999), PacketDescriptor::Unknown);
    }
}
