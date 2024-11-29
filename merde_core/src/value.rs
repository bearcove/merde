use std::collections::HashMap;

use ordered_float::OrderedFloat;

use crate::{array::Array, map::Map, CowBytes, CowStr, IntoStatic, MerdeError, ValueType};

/// Think [`serde_json::Value`](https://docs.rs/serde_json/1.0.128/serde_json/enum.Value.html), but with a small string optimization,
/// copy-on-write strings, etc. Might include other value types later.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Value<'s> {
    I64(i64),
    U64(u64),
    Float(OrderedFloat<f64>),
    Str(CowStr<'s>),
    Bytes(CowBytes<'s>),
    Null,
    Bool(bool),
    Array(Array<'s>),
    Map(Map<'s>),
}

impl IntoStatic for Value<'_> {
    type Output = Value<'static>;

    #[inline(always)]
    fn into_static(self) -> <Self as IntoStatic>::Output {
        match self {
            Value::I64(i) => Value::I64(i),
            Value::U64(u) => Value::U64(u),
            Value::Float(f) => Value::Float(f),
            Value::Str(s) => Value::Str(s.into_static()),
            Value::Bytes(b) => Value::Bytes(b.into_static()),
            Value::Null => Value::Null,
            Value::Bool(b) => Value::Bool(b),
            Value::Array(arr) => Value::Array(arr.into_static()),
            Value::Map(map) => Value::Map(map.into_static()),
        }
    }
}

macro_rules! impl_from_for_value {
    ($ty:ty => $variant:ident, $($rest:tt)*) => {
        impl_from_for_value!($ty => $variant);
        impl_from_for_value!($($rest)*);
    };

    ($ty:ty => $variant:ident) => {
        impl<'s> From<$ty> for Value<'s> {
            fn from(v: $ty) -> Self {
                Value::$variant(v.into())
            }
        }
    };

    (,) => {};
    () => {};
}

impl_from_for_value! {
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
    // misc.
    CowStr<'s> => Str,
    CowBytes<'s> => Bytes,
}

impl From<f32> for Value<'_> {
    fn from(v: f32) -> Self {
        Value::Float((v as f64).into())
    }
}

impl From<f64> for Value<'_> {
    fn from(v: f64) -> Self {
        Value::Float(v.into())
    }
}

impl<'s> From<&'s str> for Value<'s> {
    fn from(v: &'s str) -> Self {
        Value::Str(v.into())
    }
}

impl From<String> for Value<'_> {
    fn from(v: String) -> Self {
        Value::Str(v.into())
    }
}

impl<'s> From<&'s String> for Value<'s> {
    fn from(v: &'s String) -> Self {
        Value::Str(v.as_str().into())
    }
}

impl From<()> for Value<'_> {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl From<bool> for Value<'_> {
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
    pub fn as_map(&self) -> Result<&Map<'s>, MerdeError<'static>> {
        match self {
            Value::Map(obj) => Ok(obj),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Map,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn into_map(self) -> Result<Map<'s>, MerdeError<'static>> {
        match self {
            Value::Map(obj) => Ok(obj),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Map,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn as_array(&self) -> Result<&Array<'s>, MerdeError<'static>> {
        match self {
            Value::Array(arr) => Ok(arr),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn into_array(self) -> Result<Array<'s>, MerdeError<'static>> {
        match self {
            Value::Array(arr) => Ok(arr),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Array,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn as_str(&self) -> Result<&CowStr<'s>, MerdeError<'static>> {
        match self {
            Value::Str(s) => Ok(s),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::String,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn into_str(self) -> Result<CowStr<'s>, MerdeError<'static>> {
        match self {
            Value::Str(s) => Ok(s),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::String,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> Result<&CowBytes<'s>, MerdeError<'static>> {
        match self {
            Value::Bytes(b) => Ok(b),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Bytes,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn into_bytes(self) -> Result<CowBytes<'s>, MerdeError<'static>> {
        match self {
            Value::Bytes(b) => Ok(b),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Bytes,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn as_i64(&self) -> Result<i64, MerdeError<'static>> {
        match self {
            Value::I64(n) => Ok(*n),
            Value::U64(n) if *n <= i64::MAX as u64 => Ok(*n as i64),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::I64,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn as_u64(&self) -> Result<u64, MerdeError<'static>> {
        match self {
            Value::U64(n) => Ok(*n),
            Value::I64(n) => Ok((*n).try_into().map_err(|_| MerdeError::OutOfRange)?),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::U64,
                found: self.value_type(),
            }),
        }
    }

    #[inline(always)]
    pub fn as_f64(&self) -> Result<f64, MerdeError<'static>> {
        match self {
            Value::Float(n) => Ok(n.into_inner()),
            _ => Err(MerdeError::MismatchedType {
                expected: ValueType::Float,
                found: self.value_type(),
            }),
        }
    }
}
