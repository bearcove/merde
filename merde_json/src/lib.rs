#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod deserialize;
pub use deserialize::JsonDeserializer;

mod serialize;
pub use serialize::{JsonSerializer, JsonSerializerWriter};

mod jiter_lite;

use merde_core::{
    Deserialize, DeserializeOwned, DynDeserializerExt, DynSerializerExt, MerdeError, MetastackExt,
    Serialize,
};

/// Deserialize an instance of type `T` from a string of JSON text.
pub fn from_str<'s, T>(s: &'s str) -> Result<T, MerdeError<'s>>
where
    T: Deserialize<'s>,
{
    let mut deser = JsonDeserializer::new(s);
    deser.deserialize_sync::<T>()
}

/// Deserialize an instance of type `T` from a string of JSON text,
/// and return its static variant e.g. (CowStr<'static>, etc.)
pub fn from_str_owned<T>(s: &str) -> Result<T, MerdeError<'_>>
where
    T: DeserializeOwned,
{
    let mut deser = JsonDeserializer::new(s);
    T::deserialize_owned(&mut deser).run_sync_with_metastack()
}

/// Deserialize an instance of type `T` from a byte slice of JSON text.
pub fn from_bytes<'s, T>(b: &'s [u8]) -> Result<T, MerdeError<'s>>
where
    T: Deserialize<'s>,
{
    let s = std::str::from_utf8(b)?;
    from_str(s)
}

/// Deserialize an instance of type `T` from a byte slice of JSON text,
/// and return its static variant e.g. (CowStr<'static>, etc.)
pub fn from_bytes_owned<T>(b: &[u8]) -> Result<T, MerdeError<'_>>
where
    T: DeserializeOwned,
{
    let s = std::str::from_utf8(b)?;
    from_str_owned::<T>(s)
}

/// Serialize the given data structure as a String of JSON.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, MerdeError<'static>> {
    // SAFETY: This is safe because we know that the JSON serialization
    // produced by `to_json_bytes` will always be valid UTF-8.
    let res = unsafe { String::from_utf8_unchecked(to_vec(value)?) };
    Ok(res)
}

/// Serialize as JSON to a `Vec<u8>`
pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>, MerdeError<'static>> {
    let mut v: Vec<u8> = vec![];
    {
        let mut s = JsonSerializer::new(&mut v);
        s.serialize_sync(value)?;
    }
    Ok(v)
}

/// Serialize the given data structure as JSON into the I/O stream.
pub fn to_writer(
    mut writer: &mut (dyn std::io::Write + Send),
    value: &dyn DynSerialize,
) -> Result<(), MerdeError<'static>>
where
    T: Serialize,
{
    let mut s = JsonSerializer::from_writer(&mut writer);
    s.serialize_sync(value)?;
    Ok(())
}

#[cfg(feature = "tokio")]
/// Serialize the given data structure as JSON into the Tokio I/O stream.
pub async fn to_tokio_writer<W, T>(writer: &mut W, value: &T) -> Result<(), MerdeError<'static>>
where
    W: tokio::io::AsyncWrite + Unpin,
    T: Serialize,
{
    use std::pin::Pin;

    let mut s = JsonSerializer::from_tokio_writer(Pin::new(writer));
    s.serialize(value).await?;
    Ok(())
}
