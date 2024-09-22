use crate::{CowStr, MerdeError, ValueType};

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
    fn pop(&mut self) -> Result<Event<'s>, Self::Error<'s>>;

    /// Deserialize a value of type `T`.
    #[allow(async_fn_in_trait)]
    async fn t<T: Deserializable>(&mut self) -> Result<T, Self::Error<'s>> {
        self.t_starting_with(None).await
    }

    /// Deserialize a value of type `T`, using the given event as the first event.
    #[allow(async_fn_in_trait)]
    async fn t_starting_with<T: Deserializable>(
        &mut self,
        starting_with: Option<Event<'s>>,
    ) -> Result<T, Self::Error<'s>>;
}

pub trait Deserializable: Sized {
    #[allow(async_fn_in_trait)]
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>;
}

impl Deserializable for i64 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
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

impl Deserializable for u64 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl Deserializable for i32 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl Deserializable for u32 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl Deserializable for i16 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl Deserializable for u16 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl Deserializable for i8 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl Deserializable for u8 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl Deserializable for isize {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl Deserializable for usize {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl Deserializable for bool {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        Ok(de.pop()?.into_bool()?)
    }
}

impl Deserializable for f64 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: f64 = match de.pop()? {
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

impl Deserializable for f32 {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let v: f64 = de.t().await?;
        Ok(v as f32)
    }
}

impl<T: Deserializable> Deserializable for Vec<T> {
    async fn deserialize<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s>,
    {
        let array_start = de.pop()?.into_array_start()?;
        let mut vec = if let Some(size) = array_start.size_hint {
            Vec::with_capacity(size)
        } else {
            Vec::new()
        };

        loop {
            match de.pop()? {
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
