mod cowstr;
mod deserialize;
mod error;

use std::ops::Deref;
use std::ops::DerefMut;

use ahash::HashMap;
use ahash::HashMapExt;
pub use cowstr::CowStr;
pub use deserialize::ValueDeserialize;
pub use error::MerdeError;
pub use error::ValueType;

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'s> {
    Int(i64),
    Float(f64),
    Str(CowStr<'s>),
    Null,
    Bool(bool),
    Array(Array<'s>),
    Map(Map<'s>),
}

impl<'s> From<i64> for Value<'s> {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl<'s> From<f64> for Value<'s> {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl<'s> From<CowStr<'s>> for Value<'s> {
    fn from(v: CowStr<'s>) -> Self {
        Value::Str(v)
    }
}

impl<'s> From<&'s str> for Value<'s> {
    fn from(v: &'s str) -> Self {
        Value::Str(v.into())
    }
}

impl<'s> From<String> for Value<'s> {
    fn from(v: String) -> Self {
        Value::Str(v.into())
    }
}

impl<'s> From<&'s String> for Value<'s> {
    fn from(v: &'s String) -> Self {
        Value::Str(v.as_str().into())
    }
}

impl<'s> From<()> for Value<'s> {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl<'s> From<bool> for Value<'s> {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl<'s> From<Array<'s>> for Value<'s> {
    fn from(v: Array<'s>) -> Self {
        Value::Array(v)
    }
}

impl<'s> From<Map<'s>> for Value<'s> {
    fn from(v: Map<'s>) -> Self {
        Value::Map(v)
    }
}

impl<'s> From<Vec<Value<'s>>> for Value<'s> {
    fn from(v: Vec<Value<'s>>) -> Self {
        Value::Array(Array(v))
    }
}

impl<'s> From<HashMap<CowStr<'s>, Value<'s>>> for Value<'s> {
    fn from(v: HashMap<CowStr<'s>, Value<'s>>) -> Self {
        Value::Map(Map(v))
    }
}

impl<'s> Value<'s> {
    #[inline(always)]
    pub fn as_map(&self) -> Result<&Map<'s>, MerdeError> {
        match self {
            Value::Map(obj) => Ok(obj),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Map,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn as_array(&self) -> Result<&Array<'s>, MerdeError> {
        match self {
            Value::Array(arr) => Ok(arr),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn as_str(&self) -> Result<&CowStr<'s>, MerdeError> {
        match self {
            Value::Str(s) => Ok(s),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::String,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn as_i64(&self) -> Result<i64, MerdeError> {
        match self {
            Value::Int(n) => Ok(*n),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Int,
                found: self.value_type(),
            }),
        }
    }
}

/// An array of [Value]s. Named "List" because it's a bit less
/// overloaded than "Array"
#[derive(Debug, PartialEq, Clone)]
#[repr(transparent)]
pub struct Array<'s>(Vec<Value<'s>>);

impl<'s> Array<'s> {
    pub fn new() -> Self {
        Array(Vec::new())
    }
}

impl<'s> Default for Array<'s> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> From<Vec<Value<'s>>> for Array<'s> {
    fn from(v: Vec<Value<'s>>) -> Self {
        Array(v)
    }
}

impl<'s> Deref for Array<'s> {
    type Target = Vec<Value<'s>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> DerefMut for Array<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Clone)]
#[repr(transparent)]
pub struct Map<'s>(HashMap<CowStr<'s>, Value<'s>>);

impl<'s> Map<'s> {
    pub fn new() -> Self {
        Map(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Map(HashMap::with_capacity(capacity))
    }
}

impl<'s> Default for Map<'s> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> From<HashMap<CowStr<'s>, Value<'s>>> for Map<'s> {
    fn from(v: HashMap<CowStr<'s>, Value<'s>>) -> Self {
        Map(v)
    }
}

impl<'s> Deref for Map<'s> {
    type Target = HashMap<CowStr<'s>, Value<'s>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Map<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Map<'s> {
    /// Gets a value from the object, returning an error if the key is missing.
    ///
    /// Because this method knows the key name, it transforms [MerdeError::MissingValue] into [MerdeError::MissingProperty].
    ///
    /// It does not by itself throw an error if `self.get()` returns `None`, to allow
    /// for optional fields (via the [ValueDeserialize] implementation on the [Option] type).
    pub fn must_get<T>(&self, key: impl Into<CowStr<'static>>) -> Result<T, MerdeError>
    where
        T: ValueDeserialize<'s>,
    {
        let key = key.into();
        T::from_value_ref(self.get(&key)).map_err(|e| match e {
            MerdeError::MissingValue => MerdeError::MissingProperty(key),
            _ => e,
        })
    }

    /// Removes a value from the object, returning an error if the key is missing.
    ///
    /// Because this method knows the key name, it transforms [MerdeError::MissingValue] into [MerdeError::MissingProperty].
    ///
    /// It does not by itself throw an error if `self.remove()` returns `None`, to allow
    /// for optional fields (via the [ValueDeserialize] implementation on the [Option] type).
    pub fn must_remove<T>(&mut self, key: impl Into<CowStr<'static>>) -> Result<T, MerdeError>
    where
        T: ValueDeserialize<'s>,
    {
        let key = key.into();
        T::from_value(self.remove(&key)).map_err(|e| match e {
            MerdeError::MissingValue => MerdeError::MissingProperty(key),
            _ => e,
        })
    }
}

impl<'s> Array<'s> {
    /// Gets a value from the array, returning an error if the index is out of bounds.
    ///
    /// Because this method knows the index, it transforms [MerdeError::MissingValue] into [MerdeError::IndexOutOfBounds].
    ///
    /// It does not by itself throw an error if `self.get()` returns `None`, to allow
    /// for optional fields (via the [ValueDeserialize] implementation on the [Option] type).
    pub fn must_get<T>(&self, index: usize) -> Result<T, MerdeError>
    where
        T: ValueDeserialize<'s>,
    {
        T::from_value_ref(self.get(index)).map_err(|e| match e {
            MerdeError::MissingValue => MerdeError::IndexOutOfBounds {
                index,
                len: self.len(),
            },
            _ => e,
        })
    }

    /// Pops a value from the back of the array and deserializes it
    pub fn must_pop<T>(&mut self) -> Result<T, MerdeError>
    where
        T: ValueDeserialize<'s>,
    {
        T::from_value(self.pop()).map_err(|e| match e {
            MerdeError::MissingValue => MerdeError::IndexOutOfBounds {
                index: self.len(),
                len: self.len(),
            },
            _ => e,
        })
    }
}

/// Interpret a `&Value` as an instance of type `T`. This may involve
/// more cloning than [from_value].
pub fn from_value_ref<'s, T>(value: &Value<'s>) -> Result<T, MerdeError>
where
    T: ValueDeserialize<'s>,
{
    T::from_value_ref(Some(value))
}

/// Interpret a `Value` as an instance of type `T`.
pub fn from_value<'s, T>(value: Value<'s>) -> Result<T, MerdeError>
where
    T: ValueDeserialize<'s>,
{
    T::from_value(Some(value))
}
