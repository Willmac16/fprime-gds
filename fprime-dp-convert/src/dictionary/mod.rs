//! Dictionary loading and type resolution
//!
//! This module provides functionality for loading FPrime JSON dictionaries
//! and resolving type definitions for deserialization.

mod json_loader;
mod types;

pub use json_loader::JsonDictionary;
pub use types::*;

use std::collections::HashMap;
use std::path::Path;
use crate::error::{Error, Result};
use crate::types::PrimitiveType;

/// FPrime dictionary containing all type, channel, and record definitions
#[derive(Debug, Clone)]
pub struct Dictionary {
    /// Metadata about the dictionary
    pub metadata: Metadata,

    /// Type definitions keyed by qualified name
    pub types: HashMap<String, TypeDef>,

    /// Channel definitions keyed by ID
    pub channels: HashMap<u32, ChannelDef>,

    /// Record definitions keyed by ID
    pub records: HashMap<u32, RecordDef>,

    /// Container definitions keyed by ID
    pub containers: HashMap<u32, ContainerDef>,
}

impl Dictionary {
    /// Load a dictionary from a JSON file
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json_str(&content)
    }

    /// Load a dictionary from a JSON string
    pub fn from_json_str(json: &str) -> Result<Self> {
        let raw: JsonDictionary = serde_json::from_str(json)?;
        Self::from_json_dictionary(raw)
    }

    /// Convert from parsed JSON dictionary to resolved dictionary
    fn from_json_dictionary(raw: JsonDictionary) -> Result<Self> {
        // TODO: Implement full conversion with type resolution
        Ok(Self {
            metadata: Metadata {
                project_name: raw.metadata.project_name,
                framework_version: raw.metadata.framework_version,
                dictionary_hash: None,
            },
            types: HashMap::new(),
            channels: HashMap::new(),
            records: HashMap::new(),
            containers: HashMap::new(),
        })
    }

    /// Resolve a type name to its definition
    pub fn resolve_type(&self, name: &str) -> Result<&TypeDef> {
        // Check for primitive types first
        if PrimitiveType::from_name(name).is_some() {
            // Primitive types don't have definitions in the map
            return Err(Error::UnknownType(name.to_string()));
        }

        self.types.get(name).ok_or_else(|| Error::UnknownType(name.to_string()))
    }

    /// Get a channel definition by ID
    pub fn get_channel(&self, id: u32) -> Option<&ChannelDef> {
        self.channels.get(&id)
    }

    /// Get a record definition by ID
    pub fn get_record(&self, id: u32) -> Option<&RecordDef> {
        self.records.get(&id)
    }

    /// Get a container definition by ID
    pub fn get_container(&self, id: u32) -> Option<&ContainerDef> {
        self.containers.get(&id)
    }
}

/// Dictionary metadata
#[derive(Debug, Clone)]
pub struct Metadata {
    pub project_name: String,
    pub framework_version: String,
    pub dictionary_hash: Option<String>,
}
