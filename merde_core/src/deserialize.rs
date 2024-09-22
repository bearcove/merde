// it's good to have them be explicit in here
#![allow(clippy::needless_lifetimes)]

use std::{borrow::Cow, hash::Hash, str::FromStr};

use crate::{Array, CowStr, IntoStatic, Map, MerdeError, Value, ValueType, WithLifetime};

/// Lifetime-erased version of [`ValueDeserialize`] — if you want to declare a function
/// that returns something that is deserialized from a local, that something you return
/// needs to be owned, which is really hard to express normally — that's where `OwnedValueDeserialize`
/// comes in.
///
/// For more detail, see the `return-deserialize` example.
pub trait OwnedValueDeserialize
where
    Self: Sized + 'static,
{
    fn owned_from_value_ref<'s>(value: Option<&Value<'s>>) -> Result<Self, MerdeError<'s>>;
    fn owned_from_value<'s>(value: Option<Value<'s>>) -> Result<Self, MerdeError<'s>>;
}

impl<T> OwnedValueDeserialize for T
where
    T: 'static,
    for<'s> T: WithLifetime<'s>,
    for<'s> <T as WithLifetime<'s>>::Lifetimed: ValueDeserialize<'s> + IntoStatic<Output = T>,
{
    fn owned_from_value_ref<'val, 's>(
        value: Option<&'val Value<'s>>,
    ) -> Result<Self, MerdeError<'s>> {
        let t = <T as WithLifetime<'s>>::Lifetimed::from_value_ref(value)?;
        Ok(t.into_static())
    }

    fn owned_from_value<'s>(value: Option<Value<'s>>) -> Result<Self, MerdeError<'s>> {
        let t = <T as WithLifetime<'s>>::Lifetimed::from_value(value)?;
        Ok(t.into_static())
    }
}

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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>>;

    /// Destructures a JSON value into a Rust value, while taking ownership of the [Value].
    /// A default implementation is provided, but some types may want to implement it themselves
    /// to avoid unnecessary allocations/cloning.
    #[inline(always)]
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(v) => Self::from_value_ref(Some(&v)),
            None => Self::from_value_ref(None),
        }
    }
}

impl<'s> ValueDeserialize<'s> for CowStr<'s> {
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError<'s>> {
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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        Self::from_value(value.cloned())
    }
}

impl<'s> ValueDeserialize<'s> for Cow<'s, str> {
    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Str(s)) => match s {
                CowStr::Borrowed(b) => Ok(Cow::Borrowed(b)),
                #[allow(clippy::useless_conversion)]
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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        Self::from_value(value.cloned())
    }
}

impl<'s> ValueDeserialize<'s> for u8 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        u64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for u16 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        u64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for u32 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        u64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for u64 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        i64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for i16 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        i64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for i32 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        i64::from_value_ref(value)?
            .try_into()
            .map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> ValueDeserialize<'s> for i64 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
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

impl<'s> ValueDeserialize<'s> for isize {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
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

impl<'s> ValueDeserialize<'s> for f32 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Float(f)) => Ok(*f as f32),
            Some(Value::Int(i)) => Ok(*i as f32),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Float,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s> ValueDeserialize<'s> for f64 {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Float(f)) => Ok(*f),
            Some(Value::Int(i)) => Ok(*i as f64),
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Float,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s> ValueDeserialize<'s> for bool {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        Self::from_value(value.cloned())
    }
}

impl<'s, T> ValueDeserialize<'s> for Vec<T>
where
    T: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Map(obj)) => {
                let mut map = std::collections::HashMap::new();
                for (key, val) in obj.iter() {
                    let parsed_key = K::from_str(key).map_err(|_| MerdeError::InvalidKey {
                        key: key.clone().into_static(),
                        type_name: std::any::type_name::<K>(),
                    })?;
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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
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
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        Self::from_value(value.cloned())
    }
}

impl<'s, T> ValueDeserialize<'s> for Box<T>
where
    T: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        T::from_value_ref(value).map(Box::new)
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        T::from_value(value).map(Box::new)
    }
}

impl<'s, T> ValueDeserialize<'s> for std::rc::Rc<T>
where
    T: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        T::from_value_ref(value).map(std::rc::Rc::new)
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        T::from_value(value).map(std::rc::Rc::new)
    }
}

impl<'s, T> ValueDeserialize<'s> for std::sync::Arc<T>
where
    T: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        T::from_value_ref(value).map(std::sync::Arc::new)
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        T::from_value(value).map(std::sync::Arc::new)
    }
}

impl<'s, T1> ValueDeserialize<'s> for (T1,)
where
    T1: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 1 => {
                let t1 = T1::from_value_ref(Some(&arr[0]))?;
                Ok((t1,))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 1 => {
                let t1 = T1::from_value(Some(arr.into_iter().next().unwrap()))?;
                Ok((t1,))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s, T1, T2> ValueDeserialize<'s> for (T1, T2)
where
    T1: ValueDeserialize<'s>,
    T2: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 2 => {
                let t1 = T1::from_value_ref(Some(&arr[0]))?;
                let t2 = T2::from_value_ref(Some(&arr[1]))?;
                Ok((t1, t2))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 2 => {
                let mut iter = arr.into_iter();
                let t1 = T1::from_value(Some(iter.next().unwrap()))?;
                let t2 = T2::from_value(Some(iter.next().unwrap()))?;
                Ok((t1, t2))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s, T1, T2, T3> ValueDeserialize<'s> for (T1, T2, T3)
where
    T1: ValueDeserialize<'s>,
    T2: ValueDeserialize<'s>,
    T3: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 3 => {
                let t1 = T1::from_value_ref(Some(&arr[0]))?;
                let t2 = T2::from_value_ref(Some(&arr[1]))?;
                let t3 = T3::from_value_ref(Some(&arr[2]))?;
                Ok((t1, t2, t3))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 3 => {
                let mut iter = arr.into_iter();
                let t1 = T1::from_value(Some(iter.next().unwrap()))?;
                let t2 = T2::from_value(Some(iter.next().unwrap()))?;
                let t3 = T3::from_value(Some(iter.next().unwrap()))?;
                Ok((t1, t2, t3))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s, T1, T2, T3, T4> ValueDeserialize<'s> for (T1, T2, T3, T4)
where
    T1: ValueDeserialize<'s>,
    T2: ValueDeserialize<'s>,
    T3: ValueDeserialize<'s>,
    T4: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 4 => {
                let t1 = T1::from_value_ref(Some(&arr[0]))?;
                let t2 = T2::from_value_ref(Some(&arr[1]))?;
                let t3 = T3::from_value_ref(Some(&arr[2]))?;
                let t4 = T4::from_value_ref(Some(&arr[3]))?;
                Ok((t1, t2, t3, t4))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 4 => {
                let mut iter = arr.into_iter();
                let t1 = T1::from_value(Some(iter.next().unwrap()))?;
                let t2 = T2::from_value(Some(iter.next().unwrap()))?;
                let t3 = T3::from_value(Some(iter.next().unwrap()))?;
                let t4 = T4::from_value(Some(iter.next().unwrap()))?;
                Ok((t1, t2, t3, t4))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s, T1, T2, T3, T4, T5> ValueDeserialize<'s> for (T1, T2, T3, T4, T5)
where
    T1: ValueDeserialize<'s>,
    T2: ValueDeserialize<'s>,
    T3: ValueDeserialize<'s>,
    T4: ValueDeserialize<'s>,
    T5: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 5 => {
                let t1 = T1::from_value_ref(Some(&arr[0]))?;
                let t2 = T2::from_value_ref(Some(&arr[1]))?;
                let t3 = T3::from_value_ref(Some(&arr[2]))?;
                let t4 = T4::from_value_ref(Some(&arr[3]))?;
                let t5 = T5::from_value_ref(Some(&arr[4]))?;
                Ok((t1, t2, t3, t4, t5))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 5 => {
                let mut iter = arr.into_iter();
                let t1 = T1::from_value(Some(iter.next().unwrap()))?;
                let t2 = T2::from_value(Some(iter.next().unwrap()))?;
                let t3 = T3::from_value(Some(iter.next().unwrap()))?;
                let t4 = T4::from_value(Some(iter.next().unwrap()))?;
                let t5 = T5::from_value(Some(iter.next().unwrap()))?;
                Ok((t1, t2, t3, t4, t5))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s, T1, T2, T3, T4, T5, T6> ValueDeserialize<'s> for (T1, T2, T3, T4, T5, T6)
where
    T1: ValueDeserialize<'s>,
    T2: ValueDeserialize<'s>,
    T3: ValueDeserialize<'s>,
    T4: ValueDeserialize<'s>,
    T5: ValueDeserialize<'s>,
    T6: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 6 => {
                let t1 = T1::from_value_ref(Some(&arr[0]))?;
                let t2 = T2::from_value_ref(Some(&arr[1]))?;
                let t3 = T3::from_value_ref(Some(&arr[2]))?;
                let t4 = T4::from_value_ref(Some(&arr[3]))?;
                let t5 = T5::from_value_ref(Some(&arr[4]))?;
                let t6 = T6::from_value_ref(Some(&arr[5]))?;
                Ok((t1, t2, t3, t4, t5, t6))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 6 => {
                let mut iter = arr.into_iter();
                let t1 = T1::from_value(Some(iter.next().unwrap()))?;
                let t2 = T2::from_value(Some(iter.next().unwrap()))?;
                let t3 = T3::from_value(Some(iter.next().unwrap()))?;
                let t4 = T4::from_value(Some(iter.next().unwrap()))?;
                let t5 = T5::from_value(Some(iter.next().unwrap()))?;
                let t6 = T6::from_value(Some(iter.next().unwrap()))?;
                Ok((t1, t2, t3, t4, t5, t6))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s, T1, T2, T3, T4, T5, T6, T7> ValueDeserialize<'s> for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: ValueDeserialize<'s>,
    T2: ValueDeserialize<'s>,
    T3: ValueDeserialize<'s>,
    T4: ValueDeserialize<'s>,
    T5: ValueDeserialize<'s>,
    T6: ValueDeserialize<'s>,
    T7: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 7 => {
                let t1 = T1::from_value_ref(Some(&arr[0]))?;
                let t2 = T2::from_value_ref(Some(&arr[1]))?;
                let t3 = T3::from_value_ref(Some(&arr[2]))?;
                let t4 = T4::from_value_ref(Some(&arr[3]))?;
                let t5 = T5::from_value_ref(Some(&arr[4]))?;
                let t6 = T6::from_value_ref(Some(&arr[5]))?;
                let t7 = T7::from_value_ref(Some(&arr[6]))?;
                Ok((t1, t2, t3, t4, t5, t6, t7))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 7 => {
                let mut iter = arr.into_iter();
                let t1 = T1::from_value(Some(iter.next().unwrap()))?;
                let t2 = T2::from_value(Some(iter.next().unwrap()))?;
                let t3 = T3::from_value(Some(iter.next().unwrap()))?;
                let t4 = T4::from_value(Some(iter.next().unwrap()))?;
                let t5 = T5::from_value(Some(iter.next().unwrap()))?;
                let t6 = T6::from_value(Some(iter.next().unwrap()))?;
                let t7 = T7::from_value(Some(iter.next().unwrap()))?;
                Ok((t1, t2, t3, t4, t5, t6, t7))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}

impl<'s, T1, T2, T3, T4, T5, T6, T7, T8> ValueDeserialize<'s> for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: ValueDeserialize<'s>,
    T2: ValueDeserialize<'s>,
    T3: ValueDeserialize<'s>,
    T4: ValueDeserialize<'s>,
    T5: ValueDeserialize<'s>,
    T6: ValueDeserialize<'s>,
    T7: ValueDeserialize<'s>,
    T8: ValueDeserialize<'s>,
{
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 8 => {
                let t1 = T1::from_value_ref(Some(&arr[0]))?;
                let t2 = T2::from_value_ref(Some(&arr[1]))?;
                let t3 = T3::from_value_ref(Some(&arr[2]))?;
                let t4 = T4::from_value_ref(Some(&arr[3]))?;
                let t5 = T5::from_value_ref(Some(&arr[4]))?;
                let t6 = T6::from_value_ref(Some(&arr[5]))?;
                let t7 = T7::from_value_ref(Some(&arr[6]))?;
                let t8 = T8::from_value_ref(Some(&arr[7]))?;
                Ok((t1, t2, t3, t4, t5, t6, t7, t8))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }

    fn from_value(value: Option<Value<'s>>) -> Result<Self, MerdeError> {
        match value {
            Some(Value::Array(arr)) if arr.len() == 8 => {
                let mut iter = arr.into_iter();
                let t1 = T1::from_value(Some(iter.next().unwrap()))?;
                let t2 = T2::from_value(Some(iter.next().unwrap()))?;
                let t3 = T3::from_value(Some(iter.next().unwrap()))?;
                let t4 = T4::from_value(Some(iter.next().unwrap()))?;
                let t5 = T5::from_value(Some(iter.next().unwrap()))?;
                let t6 = T6::from_value(Some(iter.next().unwrap()))?;
                let t7 = T7::from_value(Some(iter.next().unwrap()))?;
                let t8 = T8::from_value(Some(iter.next().unwrap()))?;
                Ok((t1, t2, t3, t4, t5, t6, t7, t8))
            }
            Some(v) => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: v.value_type(),
            }),
            None => Err(MerdeError::MissingValue),
        }
    }
}
