//! An experimental JSON deserializer implementation

use merde_core::{ArrayStart, CowStr, Deserializer, Event, MapStart, MerdeError};

use crate::jiter_lite::{errors::JiterError, jiter::Jiter, parse::Peek};

#[derive(Debug, Clone, PartialEq, Eq)]
enum StackItem<'s> {
    ObjectKey(Option<CowStr<'s>>),
    ObjectValue,
    ObjectEnd,
    Array(Option<Peek>),
    ArrayEnd,
}

/// A JSON deserializer
pub struct JsonDeserializer<'s> {
    source: &'s str,
    jiter: Jiter<'s>,
    stack: Vec<StackItem<'s>>,
    starter: Option<Event<'s>>,
}

impl std::fmt::Debug for JsonDeserializer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonDeserializer")
            .field("source_len", &self.source)
            .field("stack_size", &self.stack.len())
            .finish()
    }
}

impl<'s> JsonDeserializer<'s> {
    /// Construct a new JSON deserializer
    pub fn new(source: &'s str) -> Self {
        let jiter = Jiter::new(source.as_bytes());
        Self {
            source,
            jiter,
            stack: Default::default(),
            starter: None,
        }
    }
}

fn jiter_error(source: &str, err: JiterError) -> MerdeError<'_> {
    MerdeError::StringParsingError {
        format: "JSON",
        index: err.index,
        message: err.error_type.to_string(),
        source: source.into(),
    }
}

impl<'s> Deserializer<'s> for JsonDeserializer<'s> {
    async fn next(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        if let Some(ev) = self.starter.take() {
            return Ok(ev);
        }

        let peek: Option<Peek> = match self.stack.pop() {
            Some(StackItem::ObjectKey(maybe_key)) => match maybe_key {
                Some(key) => {
                    self.stack.push(StackItem::ObjectValue);
                    return Ok(Event::Str(key));
                }
                None => match self
                    .jiter
                    .next_key()
                    .map_err(|e| jiter_error(self.source, e))?
                {
                    Some(key) => {
                        self.stack.push(StackItem::ObjectValue);
                        let key = cowify(self.source.as_bytes(), key);
                        return Ok(Event::Str(key));
                    }
                    None => {
                        return Ok(Event::MapEnd);
                    }
                },
            },
            Some(StackItem::ObjectValue) => {
                self.stack.push(StackItem::ObjectKey(None));
                None
            }
            Some(StackItem::ObjectEnd) => {
                return Ok(Event::MapEnd);
            }
            Some(StackItem::Array(maybe_peek)) => match maybe_peek {
                Some(peek) => {
                    self.stack.push(StackItem::Array(None));
                    Some(peek)
                }
                None => {
                    match self
                        .jiter
                        .array_step()
                        .map_err(|e| jiter_error(self.source, e))?
                    {
                        Some(peek) => {
                            self.stack.push(StackItem::Array(None));
                            Some(peek)
                        }
                        None => {
                            return Ok(Event::ArrayEnd);
                        }
                    }
                }
            },
            Some(StackItem::ArrayEnd) => {
                return Ok(Event::ArrayEnd);
            }
            None => None,
        };

        let peek = match peek {
            Some(ev) => ev,
            None => self.jiter.peek().map_err(|e| jiter_error(self.source, e))?,
        };

        let ev = if peek == Peek::Null {
            self.jiter
                .known_null()
                .map_err(|err| jiter_error(self.source, err))?;
            Event::Null
        } else if peek == Peek::True || peek == Peek::False {
            let bool_value = self
                .jiter
                .known_bool(peek)
                .map_err(|err| jiter_error(self.source, err))?;
            Event::Bool(bool_value)
        } else if peek.is_num() {
            let num = self
                .jiter
                .known_float(peek)
                .map_err(|err| jiter_error(self.source, err))?;
            if num.fract() == 0.0 && num >= i64::MIN as f64 && num <= i64::MAX as f64 {
                Event::I64(num as i64)
            } else {
                Event::F64(num)
            }
        } else if peek == Peek::String {
            let s = self
                .jiter
                .known_str()
                .map_err(|err| jiter_error(self.source, err))?;
            let s = cowify(self.source.as_bytes(), s);
            Event::Str(s)
        } else if peek == Peek::Array {
            let peek = self
                .jiter
                .known_array()
                .map_err(|err| jiter_error(self.source, err))?;
            if let Some(peek) = peek {
                self.stack.push(StackItem::Array(Some(peek)));
            } else {
                self.stack.push(StackItem::ArrayEnd);
            }
            Event::ArrayStart(ArrayStart { size_hint: None })
        } else if peek == Peek::Object {
            let key = self
                .jiter
                .known_object()
                .map_err(|err| jiter_error(self.source, err))?;
            if let Some(key) = key {
                let key = cowify(self.source.as_bytes(), key);
                self.stack.push(StackItem::ObjectKey(Some(key)));
            } else {
                self.stack.push(StackItem::ObjectEnd);
            }
            Event::MapStart(MapStart { size_hint: None })
        } else {
            panic!("Unknown peek: {:?}", peek);
        };
        Ok(ev)
    }

    fn put_back(&mut self, ev: Event<'s>) -> Result<(), MerdeError<'s>> {
        if self.starter.is_some() {
            return Err(MerdeError::PutBackCalledTwice);
        }
        self.starter = Some(ev);
        Ok(())
    }
}

fn assert_dyn_deser<'s>() {
    fn assert_impl<'s, T: merde_core::DynDeserializer<'s>>() {}
    assert_impl::<JsonDeserializer>();
}

fn assert_dyn_deser_ext<'s>() {
    fn assert_impl<'s, T: merde_core::DynDeserializerExt<'s>>() {}
    assert_impl::<JsonDeserializer>();
}

pub(crate) fn cowify<'j>(src: &'j [u8], s: &str) -> CowStr<'j> {
    if src.as_ptr_range().contains(&s.as_ptr()) {
        CowStr::Borrowed(unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(s.as_ptr(), s.len()))
        })
    } else {
        CowStr::Owned(s.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::deserialize::cowify;

    use super::JsonDeserializer;
    use merde_core::{
        Array, CowStr, Deserialize, DynDeserializer, DynDeserializerExt as _, Event, EventType,
        Map, MerdeError,
    };
    use merde_loggingserializer::LoggingDeserializer;

    #[derive(Debug, PartialEq)]
    pub struct Sample {
        pub height: i64,
        pub kind: bool,
    }

    impl<'s> Deserialize<'s> for Sample {
        async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
            {
                let mut height: Option<i64> = None;
                let mut kind: Option<bool> = None;

                de.next().await?.into_map_start()?;

                loop {
                    match de.next().await? {
                        // many different policies are possible here
                        Event::Str(k) => match k.as_ref() {
                            "height" => {
                                height = Some(i64::deserialize(de).await?);
                            }
                            "kind" => {
                                kind = Some(bool::deserialize(de).await?);
                            }
                            _ => {
                                return Err(MerdeError::UnknownProperty(k).into());
                            }
                        },
                        Event::MapEnd => {
                            break;
                        }
                        ev => {
                            return Err(MerdeError::UnexpectedEvent {
                                got: EventType::from(&ev),
                                expected: &[EventType::Str],
                                help: None,
                            }
                            .into())
                        }
                    }
                }

                Ok(Sample {
                    height: height.ok_or_else(|| MerdeError::MissingProperty("height".into()))?,
                    kind: kind.ok_or_else(|| MerdeError::MissingProperty("kind".into()))?,
                })
            }
        }
    }

    #[test]
    fn test_deserialize() {
        let input = r#"
            [
                {
                    "height": 100,
                    "kind": true
                },
                {
                    "height": 200,
                    "kind": false
                },
                {
                    "height": 150,
                    "kind": true
                }
            ]
        "#;

        let deser = JsonDeserializer::new(input);
        let mut deser = LoggingDeserializer::new(deser);

        let samples = deser.deserialize_sync::<Vec<Sample>>().unwrap();
        assert_eq!(
            samples,
            vec![
                Sample {
                    height: 100,
                    kind: true
                },
                Sample {
                    height: 200,
                    kind: false
                },
                Sample {
                    height: 150,
                    kind: true
                }
            ]
        );

        let deser = JsonDeserializer::new(input);
        let mut deser = LoggingDeserializer::new(deser);

        let value = deser.deserialize_sync::<merde_core::Value>().unwrap();

        assert_eq!(
            value,
            Array::new()
                .with(
                    Map::new()
                        .with("height", merde_core::Value::I64(100))
                        .with("kind", merde_core::Value::Bool(true))
                )
                .with(
                    Map::new()
                        .with("height", merde_core::Value::I64(200))
                        .with("kind", merde_core::Value::Bool(false))
                )
                .with(
                    Map::new()
                        .with("height", merde_core::Value::I64(150))
                        .with("kind", merde_core::Value::Bool(true))
                )
                .into()
        );
    }

    #[test]
    fn test_cowify() {
        let src = "That's a subset!";
        let s = &src[4..8];
        assert_eq!(cowify(src.as_bytes(), s), CowStr::Borrowed(s));

        let src = "Not a subset";
        let s = "indeed not";
        assert_eq!(cowify(src.as_bytes(), s), CowStr::Owned(s.into()));
    }

    // Actually trying to use Send stuff, we run into <https://github.com/rust-lang/rust/issues/100013>
    // so this test is a no-go for now

    // #[tokio::test]
    // async fn test_cowify_async() {
    //     let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    //     let port = listener.local_addr().unwrap().port();
    //     println!("Listening on: localhost:{port}");

    //     let accepted = tokio::spawn(async move {
    //         let (mut stream, _) = listener.accept().await.unwrap();

    //         let mut buf = Vec::new();
    //         stream.read_to_end(&mut buf).await.unwrap();
    //         let json_str = String::from_utf8(buf).unwrap();
    //         let mut deserializer = JsonDeserializer::new(&json_str);
    //         let deserialized: Value = Value::deserialize(&mut deserializer).await.unwrap();
    //         println!("Deserialized value: {:?}", deserialized);

    //         deserialized.into_static()
    //     });

    //     let mut arr = Array::new();
    //     for _ in 0..10 {
    //         arr.push(Map::new().with("foo", 24).with("bar", "hiya").into());
    //     }
    //     let v = Value::from(arr);

    //     let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
    //         .await
    //         .unwrap();
    //     crate::to_tokio_writer(&mut stream, &v).await.unwrap();

    //     let deserialized = accepted.await.unwrap();
    //     assert_eq!(deserialized, v);
    // }
}
