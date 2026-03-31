//! FPrime Telemetry Converter Library
//!
//! This library provides tools for parsing FPrime telemetry packets and
//! Data Product files, converting them to native Rust types and outputting
//! to time series formats like HDF5.
//!
//! # Example
//!
//! ```no_run
//! use fprime_tlm::{Dictionary, DpContainer};
//!
//! // Load dictionary
//! let dict = Dictionary::from_json_file("dictionary.json")?;
//!
//! // Parse a data product file
//! let data = std::fs::read("data_product.bin")?;
//! let container = DpContainer::parse(&data, &dict)?;
//!
//! // Access records
//! for record in &container.records {
//!     println!("{}: {:?}", record.name, record.value);
//! }
//! # Ok::<(), fprime_tlm::Error>(())
//! ```

pub mod dictionary;
pub mod error;
pub mod hash;
pub mod output;
pub mod packets;
pub mod serial;
pub mod types;

pub use dictionary::Dictionary;
pub use error::{Error, Result};
pub use packets::dp_container::DpContainer;
pub use packets::tlm_packet::TlmPacket;
pub use types::time::TimeType;
pub use types::FprimeValue;
