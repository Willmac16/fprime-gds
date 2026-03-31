//! JSON dictionary loading
//!
//! Parses FPrime JSON dictionary format into Rust structures.

use serde::{Deserialize, Serialize};

/// Raw JSON dictionary structure (matches FPrime JSON schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDictionary {
    /// Dictionary metadata
    pub metadata: JsonMetadata,

    /// Type definitions
    #[serde(default, rename = "typeDefinitions")]
    pub type_definitions: Vec<JsonTypeDef>,

    /// Channel definitions
    #[serde(default)]
    pub channels: Vec<JsonChannel>,

    /// Record definitions (for data products)
    #[serde(default)]
    pub records: Vec<JsonRecord>,

    /// Container definitions (for data products)
    #[serde(default)]
    pub containers: Vec<JsonContainer>,

    /// Constants
    #[serde(default)]
    pub constants: Vec<JsonConstant>,
}

/// Dictionary metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonMetadata {
    #[serde(default, rename = "projectName")]
    pub project_name: String,

    #[serde(default, rename = "frameworkVersion")]
    pub framework_version: String,

    #[serde(default, rename = "dictionarySpecVersion")]
    pub dictionary_spec_version: Option<String>,
}

/// JSON type definition (union type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTypeDef {
    /// Type kind: "alias", "enum", "array", "struct"
    pub kind: String,

    /// Qualified type name
    #[serde(rename = "qualifiedName")]
    pub qualified_name: String,

    // Alias-specific
    /// Target type for aliases
    #[serde(rename = "type")]
    pub target_type: Option<JsonTypeRef>,

    // Enum-specific
    /// Representation type for enums
    #[serde(rename = "representationType")]
    pub representation_type: Option<JsonTypeRef>,

    /// Enum values
    #[serde(default)]
    pub enumerants: Vec<JsonEnumerant>,

    // Array-specific
    /// Element type for arrays
    #[serde(rename = "elementType")]
    pub element_type: Option<JsonTypeRef>,

    /// Array size
    pub size: Option<usize>,

    // Struct-specific
    /// Struct members
    #[serde(default)]
    pub members: Vec<JsonStructMember>,
}

/// Type reference in JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTypeRef {
    /// Type name
    pub name: String,

    /// Type kind (primitive, qualifiedIdentifier, etc.)
    #[serde(default)]
    pub kind: Option<String>,
}

/// Enum variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonEnumerant {
    /// Variant name
    pub name: String,

    /// Numeric value
    pub value: i64,
}

/// Struct member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonStructMember {
    /// Member name
    pub name: String,

    /// Member type
    #[serde(rename = "type")]
    pub member_type: JsonTypeRef,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// Channel definition in JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonChannel {
    /// Channel ID
    pub id: u32,

    /// Channel name
    pub name: String,

    /// Component name
    #[serde(default, rename = "componentName")]
    pub component_name: String,

    /// Channel type
    #[serde(rename = "type")]
    pub channel_type: JsonTypeRef,

    /// Format string
    #[serde(default, rename = "formatString")]
    pub format_string: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Limits
    #[serde(default)]
    pub limits: Option<JsonLimits>,
}

/// Channel limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLimits {
    #[serde(rename = "lowRed")]
    pub low_red: Option<f64>,
    #[serde(rename = "lowOrange")]
    pub low_orange: Option<f64>,
    #[serde(rename = "lowYellow")]
    pub low_yellow: Option<f64>,
    #[serde(rename = "highYellow")]
    pub high_yellow: Option<f64>,
    #[serde(rename = "highOrange")]
    pub high_orange: Option<f64>,
    #[serde(rename = "highRed")]
    pub high_red: Option<f64>,
}

/// Record definition for data products
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRecord {
    /// Record ID
    pub id: u32,

    /// Record name
    pub name: String,

    /// Record type
    #[serde(rename = "type")]
    pub record_type: JsonTypeRef,

    /// Whether this is an array record
    #[serde(default)]
    pub array: bool,
}

/// Container definition for data products
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonContainer {
    /// Container ID
    pub id: u32,

    /// Container name
    pub name: String,

    /// Component name
    #[serde(default, rename = "componentName")]
    pub component_name: String,

    /// Record IDs in this container
    #[serde(default, rename = "recordIds")]
    pub record_ids: Vec<u32>,

    /// Default priority
    #[serde(rename = "defaultPriority")]
    pub default_priority: Option<u32>,
}

/// Constant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonConstant {
    /// Constant name
    pub name: String,

    /// Constant value (as string)
    pub value: serde_json::Value,

    /// Constant type
    #[serde(rename = "type")]
    pub constant_type: Option<JsonTypeRef>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_dictionary() {
        let json = r#"{
            "metadata": {
                "projectName": "Test",
                "frameworkVersion": "3.0.0"
            }
        }"#;

        let dict: JsonDictionary = serde_json::from_str(json).unwrap();
        assert_eq!(dict.metadata.project_name, "Test");
        assert_eq!(dict.metadata.framework_version, "3.0.0");
    }

    #[test]
    fn test_parse_type_definitions() {
        let json = r#"{
            "metadata": {"projectName": "Test", "frameworkVersion": "3.0.0"},
            "typeDefinitions": [
                {
                    "kind": "enum",
                    "qualifiedName": "MyEnum",
                    "representationType": {"name": "I32"},
                    "enumerants": [
                        {"name": "VALUE_A", "value": 0},
                        {"name": "VALUE_B", "value": 1}
                    ]
                }
            ]
        }"#;

        let dict: JsonDictionary = serde_json::from_str(json).unwrap();
        assert_eq!(dict.type_definitions.len(), 1);
        assert_eq!(dict.type_definitions[0].kind, "enum");
        assert_eq!(dict.type_definitions[0].enumerants.len(), 2);
    }
}
