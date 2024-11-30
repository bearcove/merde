use std::{collections::VecDeque, future::Future, io::Write};

use merde_core::{Event, MerdeError, Serializer};

/// Something the JSON serializer can write to
pub trait JsonSerializerWriter: Send {
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

/// A wrapper around a `std::io::Write` that implements `JsonSerializerWriter`
pub struct SyncWriteWrapper<'s>(&'s mut (dyn std::io::Write + Send));

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
    #[allow(clippy::manual_async_fn)]
    fn write<'fut>(
        &'fut mut self,
        ev: Event<'fut>,
    ) -> impl Future<Output = Result<(), MerdeError<'static>>> + 'fut {
        async move {
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
    pub fn from_writer<SW: std::io::Write + Send + 'w>(
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
