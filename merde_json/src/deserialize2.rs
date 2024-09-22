//! An experimental JSON deserializer implementation

use merde_core::{
    deserialize2::{ArrayStart, Deserializable, Deserializer, Event},
    CowStr, IntoStatic,
};

use crate::{
    jiter_lite::{errors::JiterError, jiter::Jiter, parse::Peek},
    MerdeJsonError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StackItem {
    ObjectKey,
    ObjectValue,
    Array,
}

/// A JSON deserializer
pub struct JsonDeserializer<'s> {
    source: &'s str,
    jiter: Jiter<'s>,
    stack: Vec<StackItem>,
    queue: Option<Event<'s>>,
}

impl<'s> JsonDeserializer<'s> {
    /// Construct a new JSON deserializer
    pub fn new(source: &'s str) -> Self {
        let jiter = Jiter::new(source.as_bytes());
        Self {
            source,
            jiter,
            stack: Default::default(),
            queue: None,
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
    type Error = MerdeJsonError<'s>;

    fn pop(&mut self) -> Result<Event, Self::Error> {
        if let Some(ev) = self.queue.take() {
            return Ok(ev);
        }

        let peek: Option<Peek> = match self.stack.last_mut().copied() {
            Some(StackItem::ObjectKey) => match self
                .jiter
                .next_key()
                .map_err(|e| jiter_error(self.source, e))?
            {
                Some(key) => {
                    *self.stack.last_mut().unwrap() = StackItem::ObjectValue;
                    return Ok(Event::Str(key.into()));
                }
                None => {
                    // end of the object/map!
                    self.stack.pop();
                    return Ok(Event::MapEnd);
                }
            },
            Some(StackItem::ObjectValue) => None,
            Some(StackItem::Array) => match self
                .jiter
                .array_step()
                .map_err(|e| jiter_error(self.source, e))?
            {
                Some(peek) => Some(peek),
                None => {
                    // end of the array!
                    self.stack.pop();
                    return Ok(Event::ArrayEnd);
                }
            },
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
            Event::Str(s.into())
        } else if peek == Peek::Array {
            self.jiter
                .known_array()
                .map_err(|err| jiter_error(self.source, err))?;
            self.stack.push(StackItem::Array);
            Event::ArrayStart(ArrayStart { size_hint: None })
        } else if peek == Peek::Object {
            let key = self
                .jiter
                .known_object()
                .map_err(|err| jiter_error(self.source, err))?;
            self.stack.push(StackItem::ObjectKey);
            if let Some(key) = key {
                self.queue = Some(Event::Str(CowStr::from(key).into_static()))
            }
            Event::MapStart
        } else {
            panic!("Unknown peek: {:?}", peek);
        };
        Ok(ev)
    }

    async fn t<T: Deserializable>(&mut self) -> Result<T, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        future::Future,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    use super::JsonDeserializer;
    use merde_core::{
        deserialize2::{Deserializable, Deserializer, Event},
        MerdeError, ValueType,
    };

    #[derive(Debug, PartialEq)]
    pub struct Sample {
        pub height: i64,
        pub kind: bool,
    }

    impl Deserializable for Sample {
        async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error>
        where
            D: Deserializer<'s>,
        {
            let mut height: Option<i64> = None;
            let mut kind: Option<bool> = None;

            de.pop()?.into_map_start()?;

            loop {
                match de.pop()? {
                    // many different policies are possible here
                    Event::Str(k) => match k.as_ref() {
                        "age" => {
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
                        return Err(MerdeError::MismatchedType {
                            expected: ValueType::String,
                            found: ValueType::from(&ev),
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
            {
                "height": 100",
                "kind": true
            }
        "#;

        let mut deser = JsonDeserializer::new(input);
        let fut = Sample::deserialize(&mut deser);
        let fut = std::pin::pin!(fut);
        let vtable = RawWakerVTable::new(|_| todo!(), |_| {}, |_| {}, |_| {});
        let vtable = Box::leak(Box::new(vtable));
        let w = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), vtable)) };
        let mut cx = Context::from_waker(&w);
        match fut.poll(&mut cx) {
            Poll::Ready(Ok(sample)) => assert_eq!(
                sample,
                Sample {
                    height: 100,
                    kind: true
                }
            ),
            _ => panic!("poll failed"),
        }
    }
}
