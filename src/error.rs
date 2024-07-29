// -------------------------------------------------------------------------
// Error Handling and Field Type
// -------------------------------------------------------------------------

use jiter::JsonValue;

/// A content-less variant of the [JsonValue] enum, used for reporting errors, see [MerdeJsonError::MismatchedType].
#[derive(Debug)]
pub enum JsonFieldType {
    Null,
    Bool,
    Int,
    BigInt,
    Float,
    String,
    Array,
    Object,
}

#[derive(Debug)]
pub enum MerdeJsonError {
    MismatchedType {
        expected: JsonFieldType,
        found: JsonFieldType,
    },
    MissingProperty(&'static str),
    UnknownProperty(String),
    JsonError(jiter::JsonError),
    OutOfRange,
    MissingValue,
    InvalidKey,
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
