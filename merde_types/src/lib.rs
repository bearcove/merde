mod cowstr;
pub use cowstr::CowStr;

mod array;
pub use array::Array;

mod map;
pub use map::Map;

mod error;
pub use error::MerdeError;
pub use error::ValueType;

mod to_static;
pub use to_static::ToStatic;

mod value;
pub use value::Value;

mod deserialize;
pub use deserialize::ValueDeserialize;

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
