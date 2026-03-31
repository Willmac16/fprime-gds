//! Dictionary type definitions

use serde::{Deserialize, Serialize};
use indexmap::IndexMap;

/// Type definition from dictionary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TypeDef {
    /// Alias to another type
    #[serde(rename = "alias")]
    Alias { name: String, target: String },

    /// Enumeration type
    #[serde(rename = "enum")]
    Enum(EnumDef),

    /// Array type
    #[serde(rename = "array")]
    Array(ArrayDef),

    /// Structure type
    #[serde(rename = "struct")]
    Struct(StructDef),
}

/// Enumeration type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDef {
    /// Qualified type name
    pub name: String,

    /// Underlying representation type (e.g., "I32")
    pub repr_type: String,

    /// Enum variants: name -> value
    pub variants: IndexMap<String, i64>,
}

impl EnumDef {
    /// Get the string representation for a numeric value
    pub fn value_to_string(&self, value: i64) -> Option<&str> {
        self.variants
            .iter()
            .find(|(_, v)| **v == value)
            .map(|(k, _)| k.as_str())
    }

    /// Get the numeric value for a string name
    pub fn string_to_value(&self, name: &str) -> Option<i64> {
        self.variants.get(name).copied()
    }
}

/// Array type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayDef {
    /// Qualified type name
    pub name: String,

    /// Element type name
    pub element_type: String,

    /// Fixed array size
    pub size: usize,
}

/// Structure type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDef {
    /// Qualified type name
    pub name: String,

    /// Struct fields in order (name -> type)
    pub fields: IndexMap<String, FieldDef>,
}

/// Structure field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    /// Field type name
    pub type_name: String,

    /// Optional field description
    pub description: Option<String>,
}

/// Channel definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDef {
    /// Channel ID
    pub id: u32,

    /// Channel name
    pub name: String,

    /// Component name (qualified)
    pub component: String,

    /// Channel value type name
    pub type_name: String,

    /// Format string for display
    pub format_string: Option<String>,

    /// Channel description
    pub description: Option<String>,

    /// Telemetry limits
    pub limits: Option<ChannelLimits>,
}

/// Channel limits for telemetry validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelLimits {
    pub low_red: Option<f64>,
    pub low_orange: Option<f64>,
    pub low_yellow: Option<f64>,
    pub high_yellow: Option<f64>,
    pub high_orange: Option<f64>,
    pub high_red: Option<f64>,
}

/// Record definition for data products
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordDef {
    /// Record ID
    pub id: u32,

    /// Record name
    pub name: String,

    /// Record data type name
    pub type_name: String,

    /// Whether this is an array record (variable size)
    pub is_array: bool,
}

/// Container definition for data products
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDef {
    /// Container ID
    pub id: u32,

    /// Container name
    pub name: String,

    /// Component name (qualified)
    pub component: String,

    /// Records contained in this container
    pub record_ids: Vec<u32>,

    /// Default priority
    pub default_priority: Option<u32>,
}

/// Packet definition for packetized telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketDef {
    /// Packet ID
    pub id: u16,

    /// Packet name
    pub name: String,

    /// Channel IDs in this packet (in order)
    pub channel_ids: Vec<u32>,
}
