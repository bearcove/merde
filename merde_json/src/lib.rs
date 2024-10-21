#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod deserialize;
pub use deserialize::JsonDeserializer;

mod jiter_lite;

use jiter_lite::errors::JiterError;
use merde_core::{
    CowStr, Deserialize, DeserializeOwned, Deserializer, IntoStatic, MerdeError, Serialize,
    Serializer,
};

use std::{collections::VecDeque, future::Future, io::Write};

/// Something the JSON serializer can write to
pub trait JsonSerializerWriter {
    /// Extend the buffer with the given slice
    fn extend_from_slice(
        &mut self,
        slice: &[u8],
    ) -> impl Future<Output = Result<(), std::io::Error>>;
}

impl JsonSerializerWriter for &mut Vec<u8> {
    async fn extend_from_slice(&mut self, slice: &[u8]) -> Result<(), std::io::Error> {
        Vec::extend_from_slice(self, slice);
        Ok(())
    }
}

struct SyncWriteWrapper<'s>(&'s mut dyn std::io::Write);

impl<'s> JsonSerializerWriter for SyncWriteWrapper<'s> {
    async fn extend_from_slice(&mut self, slice: &[u8]) -> Result<(), std::io::Error> {
        self.0.write_all(slice)
    }
}

#[cfg(feature = "tokio")]
pub mod tokio_io {
    //! Adapter types from `tokio::io::AsyncWrite` to `JsonSerializerWriter`

    use std::pin::Pin;

    use tokio::io::AsyncWriteExt;

    /// Implements `JsonSerializerWriter` for `tokio::io::AsyncWrite`
    pub struct AsyncWriteWrapper<'s>(pub Pin<&'s mut dyn tokio::io::AsyncWrite>);

    impl super::JsonSerializerWriter for AsyncWriteWrapper<'_> {
        async fn extend_from_slice(&mut self, slice: &[u8]) -> Result<(), std::io::Error> {
            self.0.write_all(slice).await
        }
    }
}

/// Writes JSON to a `Vec<u8>`. None of its methods can fail, since it doesn't target
/// an `io::Write`. You can provide your own buffer via `JsonSerializer::from_vec`.
///
/// When you're done with the serializer, you can call `JsonSerializer::into_inner` to
/// get the buffer back.
#[derive(Default)]
pub struct JsonSerializer<W>
where
    W: JsonSerializerWriter,
{
    w: W,
    stack: VecDeque<StackFrame>,
}

enum StackFrame {
    // the next item to be written is an array element
    Array { first: bool },
    // the next item to be written is a map key
    MapKey { first: bool },
    // the next item to be written is a map value
    // (and needs a ":" before it)
    MapValue,
}

impl<W> Serializer for JsonSerializer<W>
where
    W: JsonSerializerWriter,
{
    type Error = MerdeJsonError<'static>;

    async fn write(&mut self, ev: merde_core::Event<'_>) -> Result<(), Self::Error> {
        let stack_top = self.stack.back_mut();
        if let Some(stack_top) = stack_top {
            match stack_top {
                StackFrame::Array { first } => {
                    if matches!(ev, merde_core::Event::ArrayEnd) {
                        self.w.extend_from_slice(b"]").await?;
                        self.stack.pop_back();
                        return Ok(());
                    } else if *first {
                        *first = false
                    } else {
                        self.w.extend_from_slice(b",").await?;
                    }
                }
                StackFrame::MapKey { first } => {
                    if matches!(ev, merde_core::Event::MapEnd) {
                        self.w.extend_from_slice(b"}").await?;
                        self.stack.pop_back();
                        return Ok(());
                    } else {
                        if !*first {
                            self.w.extend_from_slice(b",").await?;
                        }
                        *stack_top = StackFrame::MapValue;
                        // and then let the value write itself
                    }
                }
                StackFrame::MapValue => {
                    self.w.extend_from_slice(b":").await?;
                    *stack_top = StackFrame::MapKey { first: false };
                }
            }
        }

        match ev {
            merde_core::Event::Null => {
                self.w.extend_from_slice(b"null").await?;
            }
            merde_core::Event::Bool(b) => {
                self.w
                    .extend_from_slice(if b { b"true" } else { b"false" })
                    .await?;
            }
            merde_core::Event::I64(i) => {
                let mut buf = itoa::Buffer::new();
                self.w.extend_from_slice(buf.format(i).as_bytes()).await?;
            }
            merde_core::Event::U64(u) => {
                let mut buf = itoa::Buffer::new();
                self.w.extend_from_slice(buf.format(u).as_bytes()).await?;
            }
            merde_core::Event::F64(f) => {
                let mut buf = ryu::Buffer::new();
                self.w.extend_from_slice(buf.format(f).as_bytes()).await?;
            }
            merde_core::Event::Str(s) => {
                // slow path
                self.w.extend_from_slice(b"\"").await?;
                for c in s.chars() {
                    match c {
                        '"' => self.w.extend_from_slice(b"\\\"").await?,
                        '\\' => self.w.extend_from_slice(b"\\\\").await?,
                        '\n' => self.w.extend_from_slice(b"\\n").await?,
                        '\r' => self.w.extend_from_slice(b"\\r").await?,
                        '\t' => self.w.extend_from_slice(b"\\t").await?,
                        c if c.is_control() => {
                            let mut buf = [0u8; 6];
                            write!(&mut buf[..], "\\u{:04x}", c as u32).unwrap();
                            self.w.extend_from_slice(&buf[..6]).await?;
                        }
                        c => self.w.extend_from_slice(c.to_string().as_bytes()).await?,
                    }
                }
                self.w.extend_from_slice(b"\"").await?;
            }
            merde_core::Event::MapStart(_) => {
                self.w.extend_from_slice(b"{").await?;
                self.stack.push_back(StackFrame::MapKey { first: true });
            }
            merde_core::Event::MapEnd => {
                self.w.extend_from_slice(b"}").await?;
            }
            merde_core::Event::ArrayStart(_) => {
                self.w.extend_from_slice(b"[").await?;
                self.stack.push_back(StackFrame::Array { first: true });
            }
            merde_core::Event::ArrayEnd => {
                panic!("array end without array start");
            }
            merde_core::Event::Bytes(_) => {
                // figure out what to do with those? maybe base64, maybe an array of
                // integers? unclear. maybe it should be a serializer setting.
            }
        }
        Ok(())
    }
}

impl<W> JsonSerializer<W>
where
    W: JsonSerializerWriter,
{
    /// Uses the provided buffer as the target for serialization.
    pub fn new(w: W) -> Self {
        JsonSerializer {
            w,
            stack: Default::default(),
        }
    }
}

impl<'w> JsonSerializer<SyncWriteWrapper<'w>> {
    /// Makes a json serializer that writes to a std::io::Write
    pub fn from_writer<SW: std::io::Write + 'w>(
        w: &'w mut SW,
    ) -> JsonSerializer<SyncWriteWrapper<'w>> {
        JsonSerializer::new(SyncWriteWrapper(w))
    }
}

#[cfg(feature = "tokio")]
impl<'w> JsonSerializer<tokio_io::AsyncWriteWrapper<'w>> {
    /// Makes a json serializer that writes to a tokio::io::AsyncWrite
    pub fn from_tokio_writer<SW: tokio::io::AsyncWrite + 'w>(
        w: std::pin::Pin<&'w mut SW>,
    ) -> JsonSerializer<tokio_io::AsyncWriteWrapper<'w>> {
        JsonSerializer::new(tokio_io::AsyncWriteWrapper(w))
    }
}

/// Adds convenience methods to serializable objects to write them as JSON
pub trait JsonSerialize: Serialize + Sized {
    /// Allocate a new `Vec<u8>` and serialize self to it.
    fn to_json_bytes(&self) -> Result<Vec<u8>, MerdeJsonError<'static>> {
        let mut v: Vec<u8> = vec![];
        {
            let mut s = JsonSerializer::new(&mut v);
            s.serialize_sync(self)?;
        }
        Ok(v)
    }

    /// Serialize self to a `String`.
    fn to_json_string(&self) -> Result<String, MerdeJsonError<'static>> {
        // SAFETY: This is safe because we know that the JSON serialization
        // produced by `to_json_bytes` will always be valid UTF-8.
        let res = unsafe { String::from_utf8_unchecked(self.to_json_bytes()?) };
        Ok(res)
    }
}

impl<T> JsonSerialize for T where T: Serialize {}

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

impl<'s> MerdeJsonError<'s> {
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

impl<'s> std::fmt::Display for MerdeJsonError<'s> {
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

impl<'s> std::error::Error for MerdeJsonError<'s> {}

impl<'s> std::fmt::Debug for MerdeJsonError<'s> {
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
    deser.deserialize::<T>()
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

/// Serialize the given data structure as a String of JSON.
pub fn to_string<T: JsonSerialize>(value: &T) -> Result<String, MerdeJsonError<'static>> {
    value.to_json_string()
}

/// Serialize the given data structure as a JSON byte vector.
pub fn to_vec<T: JsonSerialize>(value: &T) -> Result<Vec<u8>, MerdeJsonError<'static>> {
    value.to_json_bytes()
}

/// Serialize the given data structure as JSON into the I/O stream.
pub fn to_writer<W, T>(
    mut writer: impl std::io::Write,
    value: &T,
) -> Result<(), MerdeJsonError<'static>>
where
    T: JsonSerialize,
{
    let bytes = value.to_json_bytes()?;
    writer.write_all(&bytes).map_err(MerdeJsonError::Io)
}
