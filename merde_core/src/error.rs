// -------------------------------------------------------------------------
// Error Handling and Field Type
// -------------------------------------------------------------------------

use crate::{deserialize::TypeHints, CowStr, EventType, IntoStatic, Value};

/// A content-less variant of the [`Value`] enum, used for reporting errors, see [`MerdeError::MismatchedType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ValueType {
    /// The value is `null`.
    Null,

    /// The value is `true` or `false`.
    Bool,

    /// The value fits in an `i64`.
    I64,

    /// The value fits in a `u64`.
    U64,

    /// The value has decimal places.
    Float,

    /// The value is a string.
    String,

    /// The value is a byte array.
    Bytes,

    /// The value is an array.
    Array,

    /// The value is a map (associating keys and values)
    Map,
}

/// A grab-bag of errors that can occur when deserializing.
/// This isn't super clean, not my proudest moment.
#[derive(Debug)]
#[non_exhaustive]
pub enum MerdeError<'s> {
    /// We expected a certain type but got a different one.
    MismatchedType {
        /// The expected type.
        expected: ValueType,

        /// The type we got.
        found: ValueType,
    },

    /// We expected an object to have a certain property, but it was missing.
    MissingProperty(CowStr<'s>),

    /// We tried to access an array index that was out of bounds.
    IndexOutOfBounds {
        /// The index we tried to access.
        index: usize,
        /// The length of the array.
        len: usize,
    },

    /// We encountered a property that we didn't expect.
    UnknownProperty(CowStr<'s>),

    /// For example, we had a `u8` field but the JSON value was bigger than `u8::MAX`.
    OutOfRange,

    /// A field was missing (but we don't know its name)
    MissingValue,

    /// While calling out to [`FromStr::from_str`](std::str::FromStr::from_str) to build a [`HashMap`](std::collections::HashMap), we got an error.
    InvalidKey {
        key: CowStr<'s>,
        type_name: &'static str,
    },

    /// While parsing a datetime, we got an error
    InvalidDateTimeValue,

    UnexpectedEvent {
        got: EventType,
        expected: TypeHints,
        help: Option<String>,
    },

    /// An I/O error occurred.
    Io(std::io::Error),

    /// An Utf8 error
    Utf8Error(std::str::Utf8Error),

    /// Error occured while parsing a string, we can format
    /// a nice error message with the source string, highlighted etc.
    StringParsingError {
        format: &'static str,
        source: CowStr<'s>,
        index: usize,
        message: String,
    },

    /// Error occured while parsing binary input, let's not show source
    /// for now.
    BinaryParsingError {
        format: &'static str,
        message: String,
    },

    /// `.put_back()` was called more than once
    PutBackCalledTwice,
}

impl MerdeError<'_> {
    pub fn eof() -> Self {
        MerdeError::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "eof",
        ))
    }
}

impl IntoStatic for MerdeError<'_> {
    type Output = MerdeError<'static>;

    fn into_static(self) -> MerdeError<'static> {
        match self {
            MerdeError::MismatchedType { expected, found } => {
                MerdeError::MismatchedType { expected, found }
            }
            MerdeError::MissingProperty(prop) => MerdeError::MissingProperty(prop.into_static()),
            MerdeError::IndexOutOfBounds { index, len } => {
                MerdeError::IndexOutOfBounds { index, len }
            }
            MerdeError::UnknownProperty(prop) => MerdeError::UnknownProperty(prop.into_static()),
            MerdeError::OutOfRange => MerdeError::OutOfRange,
            MerdeError::MissingValue => MerdeError::MissingValue,
            MerdeError::InvalidKey { key, type_name } => MerdeError::InvalidKey {
                key: key.into_static(),
                type_name,
            },
            MerdeError::InvalidDateTimeValue => MerdeError::InvalidDateTimeValue,
            MerdeError::Io(e) => MerdeError::Io(e),
            MerdeError::UnexpectedEvent {
                got,
                expected,
                help: additional,
            } => MerdeError::UnexpectedEvent {
                got,
                expected,
                help: additional,
            },
            MerdeError::Utf8Error(e) => MerdeError::Utf8Error(e),
            MerdeError::StringParsingError {
                format,
                source,
                index,
                message,
            } => MerdeError::StringParsingError {
                format,
                source: source.into_static(),
                index,
                message,
            },
            MerdeError::PutBackCalledTwice => MerdeError::PutBackCalledTwice,
            MerdeError::BinaryParsingError { format, message } => {
                MerdeError::BinaryParsingError { format, message }
            }
        }
    }
}

impl From<std::io::Error> for MerdeError<'_> {
    fn from(e: std::io::Error) -> Self {
        MerdeError::Io(e)
    }
}

impl From<std::str::Utf8Error> for MerdeError<'_> {
    fn from(e: std::str::Utf8Error) -> Self {
        MerdeError::Utf8Error(e)
    }
}

impl std::fmt::Display for MerdeError<'_> {
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
            MerdeError::InvalidKey { key, type_name } => {
                write!(
                    f,
                    "Invalid key: couldn't convert {:?} to type {}",
                    key, type_name
                )
            }
            MerdeError::InvalidDateTimeValue => {
                write!(f, "Invalid date/time value")
            }
            MerdeError::Io(e) => {
                write!(f, "I/O error: {}", e)
            }
            MerdeError::UnexpectedEvent {
                got,
                expected,
                help,
            } => {
                write!(
                    f,
                    "Unexpected event: got {got:?}, expected one of {expected:?}"
                )?;
                if let Some(help) = help.as_ref() {
                    write!(f, " {help}")?;
                }
                Ok(())
            }
            MerdeError::Utf8Error(e) => {
                write!(f, "UTF-8 Error: {}", e)
            }
            MerdeError::StringParsingError {
                format,
                source,
                index,
                message,
            } => {
                let (format, source, index) = (*format, source as &str, *index);

                writeln!(f, "{format} parsing error: \x1b[31m{message}\x1b[0m",)?;
                let context_start = index.saturating_sub(20);
                let context_end = (index + 20).min(source.len());
                let context = &source[context_start..context_end];

                write!(f, "Source: ")?;
                for (i, c) in context.char_indices() {
                    if i + context_start == index {
                        write!(f, "\x1b[48;2;255;200;200m\x1b[97m{}\x1b[0m", c)?;
                    } else {
                        write!(f, "\x1b[48;2;200;200;255m\x1b[97m{}\x1b[0m", c)?;
                    }
                }
                writeln!(f)?;
                Ok(())
            }
            MerdeError::PutBackCalledTwice => {
                write!(f, "put_back() was called twice")
            }
            MerdeError::BinaryParsingError { format, message } => {
                write!(f, "{format} parsing error: {message}")
            }
        }
    }
}

impl std::error::Error for MerdeError<'_> {}

impl Value<'_> {
    /// Returns the [ValueType] for a given [Value].
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Bool(_) => ValueType::Bool,
            Value::I64(_) => ValueType::I64,
            Value::U64(_) => ValueType::U64,
            Value::Float(_) => ValueType::Float,
            Value::Str(_) => ValueType::String,
            Value::Bytes(_) => ValueType::Bytes,
            Value::Array(_) => ValueType::Array,
            Value::Map(_) => ValueType::Map,
        }
    }
}
