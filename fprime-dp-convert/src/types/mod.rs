//! FPrime type system implementation
//!
//! This module provides Rust implementations of FPrime's type system,
//! including both primitive types and complex types like TimeType.

pub mod time;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Dynamic value type for dictionary-driven deserialization
///
/// This enum represents any value that can be deserialized from FPrime
/// binary data using dictionary type information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FprimeValue {
    /// Unsigned 8-bit integer
    U8(u8),
    /// Unsigned 16-bit integer
    U16(u16),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Unsigned 64-bit integer
    U64(u64),
    /// Signed 8-bit integer
    I8(i8),
    /// Signed 16-bit integer
    I16(i16),
    /// Signed 32-bit integer
    I32(i32),
    /// Signed 64-bit integer
    I64(i64),
    /// 32-bit floating point
    F32(f32),
    /// 64-bit floating point
    F64(f64),
    /// Boolean value
    Bool(bool),
    /// UTF-8 string
    String(String),
    /// Enumeration with name, numeric value, and string representation
    Enum {
        name: String,
        value: i64,
        repr: String,
    },
    /// Fixed-size array of values
    Array(Vec<FprimeValue>),
    /// Structure with named fields (preserves field order)
    Struct(IndexMap<String, FprimeValue>),
    /// Time value
    Time(time::TimeType),
}

impl FprimeValue {
    /// Get the value as a u64, if it's an unsigned integer type
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::U8(v) => Some(*v as u64),
            Self::U16(v) => Some(*v as u64),
            Self::U32(v) => Some(*v as u64),
            Self::U64(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as an i64, if it's a signed integer type
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::I8(v) => Some(*v as i64),
            Self::I16(v) => Some(*v as i64),
            Self::I32(v) => Some(*v as i64),
            Self::I64(v) => Some(*v),
            Self::Enum { value, .. } => Some(*value),
            _ => None,
        }
    }

    /// Get the value as an f64, if it's a floating point type
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::F32(v) => Some(*v as f64),
            Self::F64(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as a bool, if it's a boolean type
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as a string reference, if it's a string type
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get the value as an array reference, if it's an array type
    pub fn as_array(&self) -> Option<&[FprimeValue]> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get the value as a struct reference, if it's a struct type
    pub fn as_struct(&self) -> Option<&IndexMap<String, FprimeValue>> {
        match self {
            Self::Struct(s) => Some(s),
            _ => None,
        }
    }

    /// Get the type name as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::U8(_) => "U8",
            Self::U16(_) => "U16",
            Self::U32(_) => "U32",
            Self::U64(_) => "U64",
            Self::I8(_) => "I8",
            Self::I16(_) => "I16",
            Self::I32(_) => "I32",
            Self::I64(_) => "I64",
            Self::F32(_) => "F32",
            Self::F64(_) => "F64",
            Self::Bool(_) => "bool",
            Self::String(_) => "string",
            Self::Enum { .. } => "enum",
            Self::Array(_) => "array",
            Self::Struct(_) => "struct",
            Self::Time(_) => "Time",
        }
    }
}

/// Primitive type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
}

impl PrimitiveType {
    /// Get the size of this primitive type in bytes
    pub fn size(self) -> usize {
        match self {
            Self::U8 | Self::I8 | Self::Bool => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 | Self::F32 => 4,
            Self::U64 | Self::I64 | Self::F64 => 8,
        }
    }

    /// Parse a primitive type from a string name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "U8" => Some(Self::U8),
            "U16" => Some(Self::U16),
            "U32" => Some(Self::U32),
            "U64" => Some(Self::U64),
            "I8" => Some(Self::I8),
            "I16" => Some(Self::I16),
            "I32" => Some(Self::I32),
            "I64" => Some(Self::I64),
            "F32" => Some(Self::F32),
            "F64" => Some(Self::F64),
            "bool" => Some(Self::Bool),
            _ => None,
        }
    }
}
