use crate::{CowStr, MerdeError, ValueType};

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

pub trait Deserializer<'s> {
    type Error: From<MerdeError<'s>>;

    /// Get the next event from the deserializer.
    fn pop(&mut self) -> Result<Event, Self::Error>;

    /// Deserialize a value of type `T`.
    #[allow(async_fn_in_trait)]
    async fn t<T: Deserializable>(&mut self) -> Result<T, Self::Error>;
}

pub trait Deserializable: Sized {
    #[allow(async_fn_in_trait)]
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'s>;
}

impl Deserializable for i64 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'s>,
    {
        let v: i64 = match de.pop()? {
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

impl Deserializable for bool {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'s>,
    {
        match de.pop()? {
            Event::Bool(b) => Ok(b),
            ev => Err(MerdeError::MismatchedType {
                expected: ValueType::Bool,
                found: ValueType::from(&ev),
            }
            .into()),
        }
    }
}
