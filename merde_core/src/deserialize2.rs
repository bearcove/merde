use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::{Array, CowStr, Map, MerdeError, Value, ValueType};

#[derive(Debug)]
pub enum Event<'s> {
    Int(i64),
    Float(f64),
    Str(CowStr<'s>),
    Bool(bool),
    Null,
    MapStart,
    MapEnd,
    ArrayStart(ArrayStart),
    ArrayEnd,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EventType {
    Int,
    Float,
    Str,
    Bool,
    Null,
    MapStart,
    MapEnd,
    ArrayStart,
    ArrayEnd,
}

impl From<&Event<'_>> for EventType {
    fn from(event: &Event<'_>) -> Self {
        match event {
            Event::Int(_) => EventType::Int,
            Event::Float(_) => EventType::Float,
            Event::Str(_) => EventType::Str,
            Event::Bool(_) => EventType::Bool,
            Event::Null => EventType::Null,
            Event::MapStart => EventType::MapStart,
            Event::MapEnd => EventType::MapEnd,
            Event::ArrayStart(_) => EventType::ArrayStart,
            Event::ArrayEnd => EventType::ArrayEnd,
        }
    }
}

#[derive(Debug)]
pub struct ArrayStart {
    pub size_hint: Option<usize>,
}

impl<'s> Event<'s> {
    pub fn into_i64(self) -> Result<i64, MerdeError<'static>> {
        match self {
            Event::Int(i) => Ok(i),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Int,
                found: ValueType::from(&self),
            }),
        }
    }

    pub fn into_f64(self) -> Result<f64, MerdeError<'static>> {
        match self {
            Event::Float(f) => Ok(f),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Float,
                found: ValueType::from(&self),
            }),
        }
    }

    pub fn into_str(self) -> Result<CowStr<'s>, MerdeError<'static>> {
        match self {
            Event::Str(s) => Ok(s),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::String,
                found: ValueType::from(&self),
            }),
        }
    }

    pub fn into_bool(self) -> Result<bool, MerdeError<'static>> {
        match self {
            Event::Bool(b) => Ok(b),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Bool,
                found: ValueType::from(&self),
            }),
        }
    }

    pub fn into_null(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::Null => Ok(()),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Null,
                found: ValueType::from(&self),
            }),
        }
    }

    pub fn into_map_start(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::MapStart => Ok(()),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Map,
                found: ValueType::from(&self),
            }),
        }
    }

    pub fn into_map_end(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::MapEnd => Ok(()),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Map,
                found: ValueType::from(&self),
            }),
        }
    }

    pub fn into_array_start(self) -> Result<ArrayStart, MerdeError<'static>> {
        match self {
            Event::ArrayStart(array_start) => Ok(array_start),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: ValueType::from(&self),
            }),
        }
    }

    pub fn into_array_end(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::ArrayEnd => Ok(()),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: ValueType::from(&self),
            }),
        }
    }
}

impl From<&Event<'_>> for ValueType {
    fn from(value: &Event<'_>) -> Self {
        match value {
            Event::Int(_) => ValueType::Int,
            Event::Float(_) => ValueType::Float,
            Event::Str(_) => ValueType::String,
            Event::Bool(_) => ValueType::Bool,
            Event::Null => ValueType::Null,
            Event::MapStart => ValueType::Map,
            Event::ArrayStart { .. } => ValueType::Array,
            _ => panic!("Invalid event for ValueType conversion"),
        }
    }
}

pub trait Deserializer<'s>: std::fmt::Debug {
    type Error<'es>: From<MerdeError<'es>>;

    /// Get the next event from the deserializer.
    #[doc(hidden)]
    fn next(&mut self) -> Result<Event<'s>, Self::Error<'s>>;

    /// Deserialize a value of type `T`.
    #[doc(hidden)]
    #[allow(async_fn_in_trait)]
    async fn t<T: Deserializable<'s>>(&mut self) -> Result<T, Self::Error<'s>> {
        self.t_starting_with(None).await
    }

    /// Deserialize a value of type `T`, using the given event as the first event.
    #[doc(hidden)]
    #[allow(async_fn_in_trait)]
    async fn t_starting_with<T: Deserializable<'s>>(
        &mut self,
        starter: Option<Event<'s>>,
    ) -> Result<T, Self::Error<'s>>;

    /// Return a boxed version of `Self::t_starting_with`, useful to avoid
    /// future sizes becoming infinite, for example when deserializing Value,
    /// etc.
    #[doc(hidden)]
    fn t_starting_with_boxed<'d, T: Deserializable<'s> + 'd>(
        &'d mut self,
        starter: Option<Event<'s>>,
    ) -> Pin<Box<dyn Future<Output = Result<T, Self::Error<'s>>> + 'd>>
    where
        's: 'd,
    {
        Box::pin(self.t_starting_with(starter))
    }

    fn deserialize<T: Deserializable<'s>>(&mut self) -> Result<T, Self::Error<'s>> {
        let vtable = RawWakerVTable::new(|_| todo!(), |_| {}, |_| {}, |_| {});
        let vtable = Box::leak(Box::new(vtable));
        let w = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), vtable)) };
        let mut cx = Context::from_waker(&w);
        let fut = self.t_starting_with(None);
        let fut = std::pin::pin!(fut);
        match fut.poll(&mut cx) {
            Poll::Ready(res) => res,
            _ => unreachable!("nothing can return poll pending yet"),
        }
    }
}

pub trait Deserializable<'s>: Sized {
    #[allow(async_fn_in_trait)]
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized;
}

impl<'s> Deserializable<'s> for i64 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = match de.next()? {
            Event::Int(i) => i,
            Event::Float(f) => f as _,
            ev => {
                return Err(MerdeError::MismatchedType {
                    expected: ValueType::Int,
                    found: ValueType::from(&ev),
                }
                .into())
            }
        };
        Ok(v)
    }
}

impl<'s> Deserializable<'s> for u64 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserializable<'s> for i32 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserializable<'s> for u32 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserializable<'s> for i16 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserializable<'s> for u16 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserializable<'s> for i8 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserializable<'s> for u8 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserializable<'s> for isize {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserializable<'s> for usize {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserializable<'s> for bool {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        Ok(de.next()?.into_bool()?)
    }
}

impl<'s> Deserializable<'s> for f64 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: f64 = match de.next()? {
            Event::Float(f) => f,
            Event::Int(i) => i as f64,
            ev => {
                return Err(MerdeError::MismatchedType {
                    expected: ValueType::Float,
                    found: ValueType::from(&ev),
                }
                .into())
            }
        };
        Ok(v)
    }
}

impl<'s> Deserializable<'s> for f32 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: f64 = de.t().await?;
        Ok(v as f32)
    }
}

impl<'s, T: Deserializable<'s>> Deserializable<'s> for Vec<T> {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let array_start = de.next()?.into_array_start()?;
        let mut vec = if let Some(size) = array_start.size_hint {
            Vec::with_capacity(size)
        } else {
            Vec::new()
        };

        loop {
            match de.next()? {
                Event::ArrayEnd => break,
                ev => {
                    let item: T = de.t_starting_with(Some(ev)).await?;
                    vec.push(item);
                }
            }
        }

        Ok(vec)
    }
}

impl<'s> Deserializable<'s> for Value<'s> {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        match de.next()? {
            Event::Int(i) => Ok(Value::Int(i)),
            Event::Float(f) => Ok(Value::Float(f)),
            Event::Str(s) => Ok(Value::Str(s)),
            Event::Bool(b) => Ok(Value::Bool(b)),
            Event::Null => Ok(Value::Null),
            Event::MapStart => {
                let mut map = Map::new();
                loop {
                    match de.next()? {
                        Event::MapEnd => break,
                        Event::Str(key) => {
                            let value: Value = de.t_starting_with_boxed(None).await?;
                            map.insert(key, value);
                        }
                        ev => {
                            return Err(MerdeError::UnexpectedEvent {
                                got: EventType::from(&ev),
                                expected: &[EventType::Str, EventType::MapEnd],
                            }
                            .into())
                        }
                    }
                }
                Ok(Value::Map(map))
            }
            Event::ArrayStart(_) => {
                let mut vec = Array::new();
                loop {
                    match de.next()? {
                        Event::ArrayEnd => break,
                        ev => {
                            let item: Value = de.t_starting_with_boxed(Some(ev)).await?;
                            vec.push(item);
                        }
                    }
                }
                Ok(Value::Array(vec))
            }
            ev => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&ev),
                expected: &[
                    EventType::Int,
                    EventType::Float,
                    EventType::Str,
                    EventType::Bool,
                    EventType::Null,
                    EventType::MapStart,
                    EventType::ArrayStart,
                ],
            }
            .into()),
        }
    }
}
