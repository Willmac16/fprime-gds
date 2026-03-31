//! FPrime packet format implementations
//!
//! This module provides parsers for FPrime packet formats including
//! Data Product containers and telemetry packets.

pub mod descriptor;
pub mod dp_container;
pub mod tlm_packet;

pub use descriptor::PacketDescriptor;
pub use dp_container::DpContainer;
pub use tlm_packet::TlmPacket;
