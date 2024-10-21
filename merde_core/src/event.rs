use std::borrow::Cow;

use crate::{CowBytes, CowStr, MerdeError};

#[derive(Debug)]
pub enum Event<'s> {
    I64(i64),
    U64(u64),
    F64(f64),
    Str(CowStr<'s>),
    Bytes(CowBytes<'s>),
    Bool(bool),
    Null,
    MapStart(MapStart),
    MapEnd,
    ArrayStart(ArrayStart),
    ArrayEnd,
}

macro_rules! impl_from_for_event {
    ($ty:ty => $variant:ident, $($rest:tt)*) => {
        impl_from_for_event!($ty => $variant);
        impl_from_for_event!($($rest)*);
    };

    ($ty:ty => $variant:ident) => {
        impl From<$ty> for Event<'_> {
            fn from(v: $ty) -> Self {
                Event::$variant(v.into())
            }
        }
    };

    (,) => {};
    () => {};
}

impl_from_for_event! {
    // signed
    i8 => I64,
    i16 => I64,
    i32 => I64,
    i64 => I64,
    // unsigned
    u8 => U64,
    u16 => U64,
    u32 => U64,
    u64 => U64,
    // floats
    f32 => F64,
    f64 => F64,
    // misc.
    bool => Bool,
}

impl From<isize> for Event<'_> {
    fn from(v: isize) -> Self {
        Event::I64(i64::try_from(v).unwrap())
    }
}

impl From<usize> for Event<'_> {
    fn from(v: usize) -> Self {
        Event::U64(u64::try_from(v).unwrap())
    }
}

impl<'s> From<&'s str> for Event<'s> {
    fn from(v: &'s str) -> Self {
        Event::Str(v.into())
    }
}

impl<'s> From<String> for Event<'s> {
    fn from(v: String) -> Self {
        Event::Str(v.into())
    }
}

impl<'s> From<Cow<'s, str>> for Event<'s> {
    fn from(v: Cow<'s, str>) -> Self {
        Event::Str(v.into())
    }
}

impl<'s> From<&'s [u8]> for Event<'s> {
    fn from(b: &'s [u8]) -> Self {
        Event::Bytes(b.into())
    }
}

impl<'s> From<Vec<u8>> for Event<'s> {
    fn from(v: Vec<u8>) -> Self {
        Event::Bytes(v.into())
    }
}

impl<'s> From<CowBytes<'s>> for Event<'s> {
    fn from(b: CowBytes<'s>) -> Self {
        Event::Bytes(b)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EventType {
    I64,
    U64,
    Float,
    Str,
    Bytes,
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
            Event::I64(_) => EventType::I64,
            Event::U64(_) => EventType::U64,
            Event::F64(_) => EventType::Float,
            Event::Str(_) => EventType::Str,
            Event::Bytes(_) => EventType::Bytes,
            Event::Bool(_) => EventType::Bool,
            Event::Null => EventType::Null,
            Event::MapStart(_) => EventType::MapStart,
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

#[derive(Debug)]
pub struct MapStart {
    pub size_hint: Option<usize>,
}

impl<'s> Event<'s> {
    pub fn into_i64(self) -> Result<i64, MerdeError<'static>> {
        match self {
            Event::I64(i) => Ok(i),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::I64],
            }),
        }
    }

    pub fn into_u64(self) -> Result<u64, MerdeError<'static>> {
        match self {
            Event::U64(u) => Ok(u),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::U64],
            }),
        }
    }

    pub fn into_f64(self) -> Result<f64, MerdeError<'static>> {
        match self {
            Event::F64(f) => Ok(f),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::Float],
            }),
        }
    }

    pub fn into_str(self) -> Result<CowStr<'s>, MerdeError<'static>> {
        match self {
            Event::Str(s) => Ok(s),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::Str],
            }),
        }
    }

    pub fn into_bytes(self) -> Result<CowBytes<'s>, MerdeError<'static>> {
        match self {
            Event::Bytes(b) => Ok(b),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::Bytes],
            }),
        }
    }

    pub fn into_bool(self) -> Result<bool, MerdeError<'static>> {
        match self {
            Event::Bool(b) => Ok(b),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::Bool],
            }),
        }
    }

    pub fn into_null(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::Null => Ok(()),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::Null],
            }),
        }
    }

    pub fn into_map_start(self) -> Result<MapStart, MerdeError<'static>> {
        match self {
            Event::MapStart(ms) => Ok(ms),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::MapStart],
            }),
        }
    }

    pub fn into_map_end(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::MapEnd => Ok(()),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::MapEnd],
            }),
        }
    }

    pub fn into_array_start(self) -> Result<ArrayStart, MerdeError<'static>> {
        match self {
            Event::ArrayStart(array_start) => Ok(array_start),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::ArrayStart],
            }),
        }
    }

    pub fn into_array_end(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::ArrayEnd => Ok(()),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::ArrayEnd],
            }),
        }
    }
}
