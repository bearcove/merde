use std::{borrow::Cow, sync::Arc};

use jiter::{JsonValue, LazyIndexMap};
use smallvec::SmallVec;

pub(crate) fn shorten_jsonvalue_lifetime<'longer, 'shorter>(
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
        JsonValue::Array(a) => JsonValue::Array(shorten_jsonarray_lifetime(a)),
        JsonValue::Object(o) => JsonValue::Object(shorten_jsonobject_lifetime(o)),
    }
}

pub(crate) fn shorten_jsonarray_lifetime<'longer, 'shorter>(
    a: Arc<SmallVec<[JsonValue<'longer>; 8]>>,
) -> Arc<SmallVec<[JsonValue<'shorter>; 8]>>
where
    'longer: 'shorter,
{
    let mut new_a: SmallVec<[JsonValue<'_>; 8]> = Default::default();
    for v in a.iter() {
        new_a.push(shorten_jsonvalue_lifetime(v.clone()));
    }
    Arc::new(new_a)
}

pub(crate) fn shorten_jsonobject_lifetime<'longer, 'shorter>(
    o: Arc<LazyIndexMap<Cow<'longer, str>, JsonValue<'longer>>>,
) -> Arc<LazyIndexMap<Cow<'shorter, str>, JsonValue<'shorter>>>
where
    'longer: 'shorter,
{
    let mut new_o: LazyIndexMap<Cow<'_, str>, JsonValue<'_>> = Default::default();
    for (k, v) in o.iter() {
        new_o.insert(k.clone(), shorten_jsonvalue_lifetime(v.clone()));
    }
    Arc::new(new_o)
}
