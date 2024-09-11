use std::{borrow::Cow, hash::Hash, str::FromStr};

use crate::{Array, CowStr, Map, MerdeError, Value, ValueType};

/// Types that can be deserialized from a [`Value`].
///
/// Implementations are provided for primitive types, strings, arrays,
/// [`HashMap`], [`Option`], etc.
///
/// There is no facility for "parsing strings as numbers". However, this
/// implementation does support numbers that are too big to fit (precisely) in
/// an `f64`, ie. integers larger than 2**53.
///
/// A field of type `HashMap<K, V>` or `Vec<T>` is required! If you want to make it optional,
/// wrap it in an `Option<T>` explicitly, e.g. `Option<HashMap<K, V>>` or `Option<Vec<T>>`.
pub trait ValueDeserialize<'s>
where
    Self: Sized,
{
    /// Destructures a [Value] into a more structured type
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError>;

    /// Destructures a JSON value into a Rust value, while taking ownership of the [Value].
    /// A default implementation is provided, but some types may want to implement it themselves
    /// to avoid unnecessary allocations/cloning.
    #[inline(always)]
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(v) => Self::from_value_ref(Some(&v)),
            None => Self::from_value_ref(None),
        }
    }
}

impl<'s> ValueDeserialize<'s> for CowStr<'s> {
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Str(s)) => Ok(s),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::String,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    #[inline(always)]
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        Self::from_value(value.cloned())
    }
}

impl<'s> ValueDeserialize<'s> for Cow<'s, str> {
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Str(s)) => match s {
                CowStr::Borrowed(b) => Ok(Cow::Borrowed(b)),
                CowStr::Owned(o) => Ok(Cow::Owned(o.into())),
            },
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::String,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    #[inline(always)]
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        Self::from_value(value.cloned())
    }
}

impl<'s> ValueDeserialize<'s> for String {
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Str(s)) => Ok(s.into()),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::String,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    #[inline(always)]
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        Self::from_value(value.cloned())
    }
}

impl<'s> ValueDeserialize<'s> for u8 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        u64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for u16 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        u64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for u32 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        u64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for u64 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Int(n)) => (*n).try_into().map_err(|_| MerdeError::OutOfRange),
            Some(Value::Float(f)) => Ok((*f).round() as u64),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Int,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s> ValueDeserialize<'s> for i8 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        i64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for i16 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        i64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for i32 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        i64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for i64 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Int(n)) => Ok(*n),
            Some(Value::Float(f)) => Ok((*f).round() as i64),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Int,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s> ValueDeserialize<'s> for usize {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Int(n)) => (*n).try_into().map_err(|_| MerdeError::OutOfRange),
            Some(Value::Float(f)) => ((*f).round() as i64)
                .try_into()
                .map_err(|_| MerdeError::OutOfRange),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Int,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s> ValueDeserialize<'s> for bool {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Bool(b)) => Ok(*b),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Bool,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s, T> ValueDeserialize<'s> for Option<T>
where
    T: ValueDeserialize<'s>,
{
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Null) => Ok(None),
            Some(v) => T::from_value(Some(v)).map(Some),
            None => Ok(None),
        }
    }

    #[inline(always)]
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        Self::from_value(value.cloned())
    }
}

impl<'s, T> ValueDeserialize<'s> for Vec<T>
where
    T: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Array(arr)) => arr
                .iter()
                .map(|item| T::from_value_ref(Some(item)))
                .collect(),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s, K, V> ValueDeserialize<'s> for std::collections::HashMap<K, V>
where
    K: FromStr + Eq + Hash + 's,
    V: ValueDeserialize<'s>,
    K::Err: std::fmt::Debug,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Map(obj)) => {
                let mut map = std::collections::HashMap::new();
                for (key, val) in obj.iter() {
                    let parsed_key = K::from_str(key).map_err(|_| MerdeError::InvalidKey)?;
                    let parsed_value = V::from_value_ref(Some(val))?;
                    map.insert(parsed_key, parsed_value);
                }
                Ok(map)
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Map,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s> ValueDeserialize<'s> for Value<'s> {
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(value) => Ok(value),
            None => Err(MerdeError::MissingValue),
        }
    }

    #[inline(always)]
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        Self::from_value(value.cloned())
    }
}

impl<'s> ValueDeserialize<'s> for Array<'s> {
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Array(arr)) => Ok(arr),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    #[inline(always)]
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        Self::from_value(value.cloned())
    }
}

impl<'s> ValueDeserialize<'s> for Map<'s> {
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Map(obj)) => Ok(obj),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Map,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    #[inline(always)]
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        Self::from_value(value.cloned())
    }
}
