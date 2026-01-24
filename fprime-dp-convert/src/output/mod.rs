//! Output format writers
//!
//! This module provides writers for various output formats including
//! HDF5, JSON, and CSV.

#[cfg(feature = "hdf5")]
pub mod hdf5;

pub mod json;

// Re-exports
#[cfg(feature = "hdf5")]
pub use self::hdf5::Hdf5Writer;
