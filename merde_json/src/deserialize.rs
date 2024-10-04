//! An experimental JSON deserializer implementation

use merde_core::{
    CowStr, {ArrayStart, Deserialize, Deserializer, Event},
};

use crate::{
    jiter_lite::{errors::JiterError, jiter::Jiter, parse::Peek},
    MerdeJsonError,
};

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

fn jiter_error(source: &str, err: JiterError) -> MerdeJsonError<'_> {
    MerdeJsonError::JiterError {
        err,
        source: Some(source.into()),
    }
}

impl<'s> Deserializer<'s> for JsonDeserializer<'s> {
    type Error<'es> = MerdeJsonError<'es>;

    fn next(&mut self) -> Result<Event<'s>, Self::Error<'s>> {
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
                Event::Int(num as i64)
            } else {
                Event::Float(num)
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
            Event::MapStart
        } else {
            panic!("Unknown peek: {:?}", peek);
        };
        Ok(ev)
    }

    async fn t_starting_with<T: Deserialize<'s>>(
        &mut self,
        starter: Option<Event<'s>>,
    ) -> Result<T, Self::Error<'s>> {
        if let Some(starter) = starter {
            if self.starter.is_some() {
                unreachable!("setting starter when it's already set? shouldn't happen")
            }
            self.starter = Some(starter);
        }

        // TODO: when too much stack space is used, stash this,
        // return Poll::Pending, to continue deserializing with
        // a shallower stack.

        // that's the whole trick — for now, we just recurse as usual
        T::deserialize(self).await
    }
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
    use merde_core::{Array, CowStr, Deserialize, Deserializer, Event, EventType, Map, MerdeError};
    use std::{
        future::Future,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    #[derive(Debug, PartialEq)]
    pub struct Sample {
        pub height: i64,
        pub kind: bool,
    }

    impl<'s> Deserialize<'s> for Sample {
        async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
        where
            D: Deserializer<'s> + ?Sized,
        {
            let mut height: Option<i64> = None;
            let mut kind: Option<bool> = None;

            de.next()?.into_map_start()?;

            loop {
                match de.next()? {
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

        struct LoggingDeserializer<'s, I>
        where
            I: Deserializer<'s>,
        {
            inner: I,
            starter: Option<Event<'s>>,
        }

        impl<'s, I> std::fmt::Debug for LoggingDeserializer<'s, I>
        where
            I: Deserializer<'s>,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("LoggingDeserializer")
                    .field("inner", &self.inner)
                    .finish()
            }
        }

        impl<'s, I> LoggingDeserializer<'s, I>
        where
            I: Deserializer<'s>,
        {
            fn new(inner: I) -> Self {
                Self {
                    inner,
                    starter: None,
                }
            }
        }

        impl<'s, I> Deserializer<'s> for LoggingDeserializer<'s, I>
        where
            I: Deserializer<'s>,
        {
            type Error<'es> = I::Error<'es>;

            fn next(&mut self) -> Result<Event<'s>, Self::Error<'s>> {
                if let Some(ev) = self.starter.take() {
                    eprintln!("> (from starter) {:?}", ev);
                    return Ok(ev);
                }

                let ev = self.inner.next()?;
                eprintln!("> {:?}", ev);
                Ok(ev)
            }

            async fn t_starting_with<T: Deserialize<'s>>(
                &mut self,
                starter: Option<Event<'s>>,
            ) -> Result<T, Self::Error<'s>> {
                if let Some(starter) = starter {
                    if self.starter.is_some() {
                        unreachable!("setting starter when it's already set? shouldn't happen")
                    }
                    self.starter = Some(starter);
                }

                T::deserialize(self).await
            }
        }

        let mut deser = LoggingDeserializer::new(deser);

        let fut = deser.t::<Vec<Sample>>();
        let fut = std::pin::pin!(fut);
        let vtable = RawWakerVTable::new(|_| todo!(), |_| {}, |_| {}, |_| {});
        let vtable = Box::leak(Box::new(vtable));
        let w = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), vtable)) };
        let mut cx = Context::from_waker(&w);
        match fut.poll(&mut cx) {
            Poll::Ready(res) => {
                let samples = res.unwrap();
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
            }
            _ => panic!("returned poll pending for some reason?"),
        }

        let mut deser = JsonDeserializer::new(input);
        let fut = deser.t::<merde_core::Value>();
        let fut = std::pin::pin!(fut);
        let vtable = RawWakerVTable::new(|_| todo!(), |_| {}, |_| {}, |_| {});
        let vtable = Box::leak(Box::new(vtable));
        let w = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), vtable)) };
        let mut cx = Context::from_waker(&w);
        match fut.poll(&mut cx) {
            Poll::Ready(res) => {
                let value = res.unwrap();
                eprintln!("value = {:#?}", value);

                assert_eq!(
                    value,
                    Array::new()
                        .with(
                            Map::new()
                                .with("height", merde_core::Value::Int(100))
                                .with("kind", merde_core::Value::Bool(true))
                        )
                        .with(
                            Map::new()
                                .with("height", merde_core::Value::Int(200))
                                .with("kind", merde_core::Value::Bool(false))
                        )
                        .with(
                            Map::new()
                                .with("height", merde_core::Value::Int(150))
                                .with("kind", merde_core::Value::Bool(true))
                        )
                        .into()
                );
            }
            _ => panic!("returned poll pending for some reason?"),
        }
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
}
