use std::borrow::Cow;

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

///

pub trait JsonDeserialize3<'src>
where
    Self: Sized,
{
    fn json_deserialize<'val>(
        value: Option<&'val crate::JsonValue<'src>>,
    ) -> Result<Self, crate::MerdeJsonError>;
}

impl<'src> JsonDeserialize3<'src> for Cow<'src, str> {
    fn json_deserialize<'val>(
        value: Option<&'val crate::JsonValue<'src>>,
    ) -> Result<Self, crate::MerdeJsonError> {
        let value = value.ok_or(crate::MerdeJsonError::MissingValue)?;
        match value {
            crate::JsonValue::Str(s) => Ok(s.clone()),
            _ => Err(crate::MerdeJsonError::MismatchedType {
                expected: crate::JsonFieldType::String,
                // not true
                found: crate::JsonFieldType::Null,
            }),
        }
    }
}
