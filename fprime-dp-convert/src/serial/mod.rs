//! Serialization infrastructure
//!
//! This module provides the core serialization primitives matching
//! FPrime's `Fw::SerialBuffer` implementation.

mod buffer;
mod endian;
mod traits;

pub use buffer::SerialBuffer;
pub use endian::Endianness;
pub use traits::{Deserialize, Serialize};
