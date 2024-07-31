// -------------------------------------------------------------------------
// Error Handling and Field Type
// -------------------------------------------------------------------------

use jiter::JsonValue;

/// A content-less variant of the [JsonValue] enum, used for reporting errors, see [MerdeJsonError::MismatchedType].
#[derive(Debug)]
#[non_exhaustive]
pub enum JsonFieldType {
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
    Object,
}

/// A grab-bag of errors that can occur when deserializing JSON.
/// This isn't super clean, not my proudest moment.
#[derive(Debug)]
#[non_exhaustive]
pub enum MerdeJsonError {
    /// We expected a certain type but got a different one.
    ///
    /// Note that the default implementations of [crate::JsonDeserialize] have tolerances:
    /// if we expect a `u32` but get a floating-point number, we'll round it.
    MismatchedType {
        /// The expected type.
        expected: JsonFieldType,

        /// The type we got.
        found: JsonFieldType,
    },

    /// We expected an object to have a certain property, but it was missing.
    MissingProperty(&'static str),

    /// We tried to access an array index that was out of bounds.
    IndexOutOfBounds {
        /// The index we tried to access.
        index: usize,
        /// The length of the array.
        len: usize,
    },

    /// We encountered a property that we didn't expect.
    UnknownProperty(String),

    /// We encountered an error in the underlying JSON parser.
    JsonError(jiter::JsonError),

    /// For example, we had a `u8` field but the JSON value was bigger than `u8::MAX`.
    OutOfRange,

    /// A field was missing (but we don't know its name)
    MissingValue,

    /// While calling out to [FromStr::from_str](std::str::FromStr::from_str) to build a [HashMap](std::collections::HashMap), we got an error.
    InvalidKey,

    /// While parsing a [time::Date] or [time::Time], we got an error.
    InvalidDateTimeValue,
}

impl From<jiter::JsonError> for MerdeJsonError {
    fn from(e: jiter::JsonError) -> Self {
        MerdeJsonError::JsonError(e)
    }
}

impl std::fmt::Display for MerdeJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MerdeJsonError::MismatchedType { expected, found } => {
                write!(f, "Expected {:?}, found {:?}", expected, found)
            }
            MerdeJsonError::MissingProperty(prop) => {
                write!(f, "Missing property: {}", prop)
            }
            MerdeJsonError::IndexOutOfBounds { index, len: length } => {
                write!(
                    f,
                    "Index out of bounds: index {} is not valid for length {}",
                    index, length
                )
            }
            MerdeJsonError::UnknownProperty(prop) => {
                write!(f, "Unknown property: {}", prop)
            }
            MerdeJsonError::JsonError(e) => {
                write!(f, "JsonError: {}", e)
            }
            MerdeJsonError::OutOfRange => {
                write!(f, "Value is out of range")
            }
            MerdeJsonError::MissingValue => {
                write!(f, "Missing value")
            }
            MerdeJsonError::InvalidKey => {
                write!(f, "Invalid key")
            }
            MerdeJsonError::InvalidDateTimeValue => {
                write!(f, "Invalid date/time value")
            }
        }
    }
}

impl std::error::Error for MerdeJsonError {}

impl From<&JsonValue<'_>> for JsonFieldType {
    fn from(value: &JsonValue) -> Self {
        match value {
            JsonValue::Null => JsonFieldType::Null,
            JsonValue::Bool(_) => JsonFieldType::Bool,
            JsonValue::Int(_) => JsonFieldType::Int,
            JsonValue::BigInt(_) => JsonFieldType::BigInt,
            JsonValue::Float(_) => JsonFieldType::Float,
            JsonValue::Str(_) => JsonFieldType::String,
            JsonValue::Array(_) => JsonFieldType::Array,
            JsonValue::Object(_) => JsonFieldType::Object,
        }
    }
}
