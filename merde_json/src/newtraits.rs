use std::{borrow::Cow, collections::HashMap, sync::Arc};

use jiter::{JsonObject, JsonValue, LazyIndexMap};
use smallvec::SmallVec;

use crate::JsonValueExt;

pub enum JsonValue2<'src> {
    Str(Cow<'src, str>),
}

fn covariant_jsonvalue2<'longer, 'shorter>(t: JsonValue2<'longer>) -> JsonValue2<'shorter>
where
    'longer: 'shorter,
{
    t
}

fn covariant_jsonvalue<'longer, 'shorter>(
    t: crate::JsonValue<'longer>,
) -> crate::JsonValue<'shorter>
where
    'longer: 'shorter,
{
    t
}

fn covariant_jsonvalue_clone<'longer, 'shorter>(
    t: crate::JsonValue<'longer>,
) -> crate::JsonValue<'shorter>
where
    'longer: 'shorter,
{
    match t {
        JsonValue::Null => JsonValue::Null,
        JsonValue::Bool(b) => JsonValue::Bool(b),
        JsonValue::Int(i) => JsonValue::Int(i),
        JsonValue::BigInt(bi) => JsonValue::BigInt(bi),
        JsonValue::Float(f) => JsonValue::Float(f),
        JsonValue::Str(s) => JsonValue::Str(s),
        JsonValue::Array(a) => {
            let mut new_a: SmallVec<[JsonValue<'_>; 8]> = Default::default();
            for v in a.iter() {
                new_a.push(covariant_jsonvalue_clone(v.clone()));
            }
            JsonValue::Array(Arc::new(new_a))
        }
        JsonValue::Object(o) => {
            let mut new_o: LazyIndexMap<Cow<'_, str>, JsonValue<'_>> = Default::default();
            for (k, v) in o.iter() {
                new_o.insert(k.clone(), covariant_jsonvalue_clone(v.clone()));
            }
            JsonValue::Object(Arc::new(new_o))
        }
    }
}

fn covariant_jsonobject<'longer, 'shorter>(
    t: crate::JsonObject<'longer>,
) -> crate::JsonObject<'shorter>
where
    'longer: 'shorter,
{
    t
}

fn covariant_jsonarray<'longer, 'shorter>(
    t: crate::JsonArray<'longer>,
) -> crate::JsonArray<'shorter>
where
    'longer: 'shorter,
{
    t
}

fn covariant_lazyindexmap<'longer, 'shorter, K, V>(
    t: jiter::LazyIndexMap<Cow<'longer, str>, String>,
) -> jiter::LazyIndexMap<Cow<'shorter, str>, String>
where
    'longer: 'shorter,
{
    t
}

fn covariant_smallvec<'longer, 'shorter, T>(
    t: SmallVec<[Cow<'longer, str>; 8]>,
) -> SmallVec<[Cow<'shorter, str>; 8]>
where
    'longer: 'shorter,
{
    t
}

fn covariant_smallvec_maybe<'longer, 'shorter, T>(
    t: SmallVec<[Cow<'longer, str>; 8]>,
) -> SmallVec<[Cow<'shorter, str>; 8]>
where
    'longer: 'shorter,
{
    t.into_iter()
        .map(|s| match s {
            Cow::Borrowed(s) => Cow::Borrowed(s),
            Cow::Owned(s) => Cow::Owned(s.clone()),
        })
        .collect()
}

// -------------------------------------------------------------------------

pub trait JsonDeserialize2<'src>
where
    Self: Sized,
{
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue2<'src>>,
    ) -> Result<Self, crate::MerdeJsonError>;
}

impl<'src> JsonDeserialize2<'src> for Cow<'src, str> {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue2<'src>>,
    ) -> Result<Self, crate::MerdeJsonError> {
        let value = value.ok_or(crate::MerdeJsonError::MissingValue)?;
        match value {
            JsonValue2::Str(s) => Ok(s.clone()),
            _ => Err(crate::MerdeJsonError::MismatchedType {
                expected: crate::JsonFieldType::String,
                // not true
                found: crate::JsonFieldType::Null,
            }),
        }
    }
}

// -------------------------------------------------------------------------

pub trait JsonDeserialize3<'src>
where
    Self: Sized,
{
    fn json_deserialize<'val>(
        value: Option<&'val crate::JsonValue<'src>>,
    ) -> Result<Self, crate::MerdeJsonError>;
}

impl<'src> JsonDeserialize3<'src> for HashMap<Cow<'src, str>, JsonValue<'src>> {
    fn json_deserialize<'val>(
        value: Option<&'val crate::JsonValue<'src>>,
    ) -> Result<Self, crate::MerdeJsonError> {
        let value = value.ok_or(crate::MerdeJsonError::MissingValue)?;
        match value {
            crate::JsonValue::Object(s) => {
                let mut map = HashMap::new();
                for (key, val) in s.iter() {
                    let parsed_key = key.clone();
                    let parsed_value = val.clone();
                    map.insert(parsed_key, parsed_value);
                }
                Ok(map)
            }
            _ => Err(crate::MerdeJsonError::MismatchedType {
                expected: crate::JsonFieldType::String,
                // not true
                found: crate::JsonFieldType::Null,
            }),
        }
    }
}
