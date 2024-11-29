#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod deserialize;
pub use deserialize::JsonDeserializer;

mod serialize;
pub use serialize::{JsonSerializer, JsonSerializerWriter};

mod jiter_lite;

use jiter_lite::errors::JiterError;
use merde_core::{
    CowStr, Deserialize, DeserializeOwned, Deserializer, IntoStatic, MerdeError, Serialize,
    Serializer,
};
/// Unifies [MerdeError] and [JiterError] into a single type
pub enum MerdeJsonError<'s> {
    /// A [MerdeError]
    MerdeError(MerdeError<'s>),

    /// An Utf8 error
    Utf8Error(std::str::Utf8Error),

    /// A [JiterError]
    JiterError {
        /// The underlying jiter error
        err: JiterError,
        /// The JSON source, if available
        source: Option<CowStr<'s>>,
    },

    /// An I/O error that occured while writing
    Io(std::io::Error),

    /// Tried to serialize bytes to JSON
    JsonDoesNotSupportBytes,
}

impl From<std::io::Error> for MerdeJsonError<'static> {
    fn from(e: std::io::Error) -> Self {
        MerdeJsonError::Io(e)
    }
}

impl MerdeJsonError<'_> {
    /// Strip the 'source' field from the error, making it `'static`
    pub fn without_source(self) -> MerdeJsonError<'static> {
        match self {
            MerdeJsonError::MerdeError(e) => MerdeJsonError::MerdeError(e.into_static()),
            MerdeJsonError::Utf8Error(e) => MerdeJsonError::Utf8Error(e),
            MerdeJsonError::JiterError { err, source: _ } => {
                MerdeJsonError::JiterError { err, source: None }
            }
            MerdeJsonError::JsonDoesNotSupportBytes => MerdeJsonError::JsonDoesNotSupportBytes,
            MerdeJsonError::Io(e) => MerdeJsonError::Io(e),
        }
    }

    /// Converts the attached 'source' field to an owned string, making the whole error `'static`
    pub fn to_static(self) -> MerdeJsonError<'static> {
        match self {
            MerdeJsonError::MerdeError(e) => MerdeJsonError::MerdeError(e.into_static()),
            MerdeJsonError::Utf8Error(e) => MerdeJsonError::Utf8Error(e),
            MerdeJsonError::JiterError { err, source } => MerdeJsonError::JiterError {
                err,
                source: source.map(|s| s.into_static()),
            },
            MerdeJsonError::JsonDoesNotSupportBytes => MerdeJsonError::JsonDoesNotSupportBytes,
            MerdeJsonError::Io(e) => MerdeJsonError::Io(e),
        }
    }
}

impl IntoStatic for MerdeJsonError<'_> {
    type Output = MerdeJsonError<'static>;

    fn into_static(self) -> Self::Output {
        self.to_static()
    }
}

impl From<std::str::Utf8Error> for MerdeJsonError<'_> {
    fn from(e: std::str::Utf8Error) -> Self {
        MerdeJsonError::Utf8Error(e)
    }
}

impl std::fmt::Display for MerdeJsonError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MerdeJsonError::MerdeError(me) => write!(f, "Merde Error: {}", me),
            MerdeJsonError::Utf8Error(ue) => write!(f, "UTF-8 Error: {}", ue),
            MerdeJsonError::JiterError { err, source } => {
                writeln!(f, "JSON parsing error: \x1b[31m{}\x1b[0m", err.error_type)?;
                if let Some(source) = source {
                    let context_start = err.index.saturating_sub(20);
                    let context_end = (err.index + 20).min(source.len());
                    let context = &source[context_start..context_end];

                    write!(f, "Source: ")?;
                    for (i, c) in context.char_indices() {
                        if i + context_start == err.index {
                            write!(f, "\x1b[48;2;255;200;200m\x1b[97m{}\x1b[0m", c)?;
                        } else {
                            write!(f, "\x1b[48;2;200;200;255m\x1b[97m{}\x1b[0m", c)?;
                        }
                    }
                    writeln!(f)?;
                } else {
                    writeln!(f, "Error context: (not attached)")?;
                }
                Ok(())
            }
            MerdeJsonError::JsonDoesNotSupportBytes => {
                write!(
                    f,
                    "tried to serialize bytes to JSON (bytes are not supported)"
                )
            }
            MerdeJsonError::Io(e) => {
                write!(f, "IO Error: {}", e)
            }
        }
    }
}

impl std::error::Error for MerdeJsonError<'_> {}

impl std::fmt::Debug for MerdeJsonError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl<'s> From<MerdeError<'s>> for MerdeJsonError<'s> {
    fn from(e: MerdeError<'s>) -> Self {
        MerdeJsonError::MerdeError(e)
    }
}

/// Deserialize an instance of type `T` from a string of JSON text.
pub fn from_str<'s, T>(s: &'s str) -> Result<T, MerdeJsonError<'s>>
where
    T: Deserialize<'s>,
{
    let mut deser = JsonDeserializer::new(s);
    deser.deserialize_sync::<T>()
}

/// Deserialize an instance of type `T` from a string of JSON text,
/// and return its static variant e.g. (CowStr<'static>, etc.)
pub fn from_str_owned<T>(s: &str) -> Result<T, MerdeJsonError<'_>>
where
    T: DeserializeOwned,
{
    let mut deser = JsonDeserializer::new(s);
    T::deserialize_owned(&mut deser)
}

/// Deserialize an instance of type `T` from a byte slice of JSON text.
pub fn from_bytes<'s, T>(b: &'s [u8]) -> Result<T, MerdeJsonError<'s>>
where
    T: Deserialize<'s>,
{
    let s = std::str::from_utf8(b)?;
    from_str(s)
}

/// Deserialize an instance of type `T` from a byte slice of JSON text,
/// and return its static variant e.g. (CowStr<'static>, etc.)
pub fn from_bytes_owned<T>(b: &[u8]) -> Result<T, MerdeJsonError<'_>>
where
    T: DeserializeOwned,
{
    let s = std::str::from_utf8(b)?;
    from_str_owned(s)
}

/// Serialize the given data structure as a String of JSON.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, MerdeJsonError<'static>> {
    // SAFETY: This is safe because we know that the JSON serialization
    // produced by `to_json_bytes` will always be valid UTF-8.
    let res = unsafe { String::from_utf8_unchecked(to_vec(value)?) };
    Ok(res)
}

/// Serialize as JSON to a `Vec<u8>`
pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>, MerdeJsonError<'static>> {
    let mut v: Vec<u8> = vec![];
    {
        let mut s = JsonSerializer::new(&mut v);
        s.serialize_sync(value)?;
    }
    Ok(v)
}

/// Serialize the given data structure as JSON into the I/O stream.
pub fn to_writer<W, T>(
    mut writer: impl std::io::Write,
    value: &T,
) -> Result<(), MerdeJsonError<'static>>
where
    T: Serialize,
{
    let mut s = JsonSerializer::from_writer(&mut writer);
    s.serialize_sync(value)?;
    Ok(())
}

#[cfg(feature = "tokio")]
/// Serialize the given data structure as JSON into the Tokio I/O stream.
pub async fn to_tokio_writer<W, T>(writer: &mut W, value: &T) -> Result<(), MerdeJsonError<'static>>
where
    W: tokio::io::AsyncWrite + Unpin,
    T: Serialize,
{
    use std::pin::Pin;

    let mut s = JsonSerializer::from_tokio_writer(Pin::new(writer));
    s.serialize(value).await?;
    Ok(())
}
