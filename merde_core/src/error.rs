// -------------------------------------------------------------------------
// Error Handling and Field Type
// -------------------------------------------------------------------------

use crate::{CowStr, Value};

/// A content-less variant of the [Value] enum, used for reporting errors, see [MerdeJsonError::MismatchedType].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ValueType {
    /// The JSON value is `null`.
    Null,

    /// The JSON value is `true` or `false`.
    Bool,

    /// The JSON value fits in an `i64`.
    Int,

    /// The JSON value no longer fits in an `i64`.
    BigInt,

    /// The JSON value has decimal places.
    Float,

    /// The JSON value is a string.
    String,

    /// The JSON value is an array.
    Array,

    /// The JSON value is an object. Keys must be strings.
    Map,
}

/// A grab-bag of errors that can occur when deserializing.
/// This isn't super clean, not my proudest moment.
#[derive(Debug)]
#[non_exhaustive]
pub enum MerdeError {
    /// We expected a certain type but got a different one.
    ///
    /// Note that the default implementations of [crate::ValueDeserialize] have tolerances:
    /// if we expect a `u32` but get a floating-point number, we'll round it.
    MismatchedType {
        /// The expected type.
        expected: ValueType,

        /// The type we got.
        found: ValueType,
    },

    /// We expected an object to have a certain property, but it was missing.
    MissingProperty(CowStr<'static>),

    /// We tried to access an array index that was out of bounds.
    IndexOutOfBounds {
        /// The index we tried to access.
        index: usize,
        /// The length of the array.
        len: usize,
    },

    /// We encountered a property that we didn't expect.
    UnknownProperty(String),

    /// For example, we had a `u8` field but the JSON value was bigger than `u8::MAX`.
    OutOfRange,

    /// A field was missing (but we don't know its name)
    MissingValue,

    /// While calling out to [FromStr::from_str](std::str::FromStr::from_str) to build a [HashMap](std::collections::HashMap), we got an error.
    InvalidKey,

    /// While parsing a datetime, we got an error
    InvalidDateTimeValue,

    /// An I/O error occurred.
    Io(std::io::Error),
}

impl From<std::io::Error> for MerdeError {
    fn from(e: std::io::Error) -> Self {
        MerdeError::Io(e)
    }
}

impl std::fmt::Display for MerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MerdeError::MismatchedType { expected, found } => {
                write!(f, "Expected {:?}, found {:?}", expected, found)
            }
            MerdeError::MissingProperty(prop) => {
                write!(f, "Missing property: {}", prop)
            }
            MerdeError::IndexOutOfBounds { index, len: length } => {
                write!(
                    f,
                    "Index out of bounds: index {} is not valid for length {}",
                    index, length
                )
            }
            MerdeError::UnknownProperty(prop) => {
                write!(f, "Unknown property: {}", prop)
            }
            MerdeError::OutOfRange => {
                write!(f, "Value is out of range")
            }
            MerdeError::MissingValue => {
                write!(f, "Missing value")
            }
            MerdeError::InvalidKey => {
                write!(f, "Invalid key")
            }
            MerdeError::InvalidDateTimeValue => {
                write!(f, "Invalid date/time value")
            }
            MerdeError::Io(e) => {
                write!(f, "I/O error: {}", e)
            }
        }
    }
}

impl std::error::Error for MerdeError {}

impl Value<'_> {
    /// Returns the [ValueType] for a given [Value].
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::Str(_) => ValueType::String,
            Value::Array(_) => ValueType::Array,
            Value::Map(_) => ValueType::Map,
        }
    }
}
