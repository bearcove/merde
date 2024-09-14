use std::collections::HashMap;

use crate::{array::Array, map::Map, CowStr, IntoStatic, MerdeError, ValueType};

/// Think [`serde_json::Value`](https://docs.rs/serde_json/1.0.128/serde_json/enum.Value.html), but with a small string optimization,
/// copy-on-write strings, etc. Might include other value types later.
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

impl IntoStatic for Value<'_> {
    type Output = Value<'static>;

    #[inline(always)]
    fn into_static(self) -> <Self as IntoStatic>::Output {
        match self {
            Value::Int(i) => Value::Int(i),
            Value::Float(f) => Value::Float(f),
            Value::Str(s) => Value::Str(s.into_static()),
            Value::Null => Value::Null,
            Value::Bool(b) => Value::Bool(b),
            Value::Array(arr) => Value::Array(arr.into_static()),
            Value::Map(map) => Value::Map(map.into_static()),
        }
    }
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
    pub fn into_map(self) -> Result<Map<'s>, MerdeError> {
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
    pub fn into_array(self) -> Result<Array<'s>, MerdeError> {
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
    pub fn into_str(self) -> Result<CowStr<'s>, MerdeError> {
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
