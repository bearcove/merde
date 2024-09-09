#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod variance;

mod error;
pub use error::*;

pub use jiter::{JsonArray, JsonObject, JsonValue};
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;
use std::str::FromStr;

/// Implemented by anything that can be deserialized from JSON:
///
/// Implementations are provided for primitive types, strings, arrays,
/// HashMap, Option, and slices of tuples (for when you don't _need_ the
/// "hash" part of the HashMap).
///
/// There is no facility for "parsing strings as numbers". However, this
/// implementation does support numbers that are too big to fit (precisely) in
/// an `f64`, ie. integers larger than 2**53.
///
/// A field of type `HashMap<K, V>` or `Vec<T>` is required! If you want to make it optional,
/// wrap it in an `Option<T>` explicitly, e.g. `Option<HashMap<K, V>>` or `Option<Vec<T>>`.
pub trait JsonDeserialize<'src>
where
    Self: Sized,
{
    /// Destructures a JSON value into a Rust value
    fn json_deserialize<'val>(value: Option<&'val JsonValue<'src>>)
        -> Result<Self, MerdeJsonError>;

    /// Destructures a JSON value into a Rust value, while taking ownership of the [JsonValue].
    /// A default implementation is provided, but some types may want to implement it themselves
    /// to avoid unnecessary allocations/cloning.
    #[inline(always)]
    fn json_deserialize_taking_ownership(
        value: Option<JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(v) => Self::json_deserialize(Some(&v)),
            None => Self::json_deserialize(None),
        }
    }
}

impl<'src> JsonDeserialize<'src> for Cow<'src, str> {
    fn json_deserialize_taking_ownership(
        value: Option<JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Str(s)) => Ok(s),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::String,
                found: JsonFieldType::for_json_value(&v),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }

    #[inline(always)]
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        Self::json_deserialize_taking_ownership(value.cloned())
    }
}

impl<'src> JsonDeserialize<'src> for String {
    fn json_deserialize_taking_ownership(
        value: Option<JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Str(s)) => Ok(s.into_owned()),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::String,
                found: JsonFieldType::for_json_value(&v),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }

    #[inline(always)]
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        Self::json_deserialize_taking_ownership(value.cloned())
    }
}

impl<'src> JsonDeserialize<'src> for u8 {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        u64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'src> JsonDeserialize<'src> for u16 {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        u64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'src> JsonDeserialize<'src> for u32 {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        u64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'src> JsonDeserialize<'src> for u64 {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Int(n)) => (*n).try_into().map_err(|_| MerdeJsonError::OutOfRange),
            Some(JsonValue::Float(f)) => Ok((*f).round() as u64),
            Some(JsonValue::BigInt(bi)) => bi.try_into().map_err(|_| MerdeJsonError::OutOfRange),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Int,
                found: JsonFieldType::for_json_value(v),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'src> JsonDeserialize<'src> for i8 {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        i64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'src> JsonDeserialize<'src> for i16 {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        i64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'src> JsonDeserialize<'src> for i32 {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        i64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'src> JsonDeserialize<'src> for i64 {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Int(n)) => Ok(*n),
            Some(JsonValue::Float(f)) => Ok((*f).round() as i64),
            Some(JsonValue::BigInt(bi)) => bi.try_into().map_err(|_| MerdeJsonError::OutOfRange),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Int,
                found: JsonFieldType::for_json_value(v),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'src> JsonDeserialize<'src> for usize {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Int(n)) => (*n).try_into().map_err(|_| MerdeJsonError::OutOfRange),
            Some(JsonValue::Float(f)) => ((*f).round() as i64)
                .try_into()
                .map_err(|_| MerdeJsonError::OutOfRange),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Int,
                found: JsonFieldType::for_json_value(v),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'src> JsonDeserialize<'src> for bool {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Bool(b)) => Ok(*b),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Bool,
                found: JsonFieldType::for_json_value(v),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'src, T> JsonDeserialize<'src> for Option<T>
where
    T: JsonDeserialize<'src>,
{
    fn json_deserialize_taking_ownership(
        value: Option<JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Null) => Ok(None),
            Some(v) => T::json_deserialize_taking_ownership(Some(v)).map(Some),
            None => Ok(None),
        }
    }

    #[inline(always)]
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        Self::json_deserialize_taking_ownership(value.cloned())
    }
}

impl<'src, T> JsonDeserialize<'src> for Vec<T>
where
    T: JsonDeserialize<'src>,
{
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Array(arr)) => arr
                .iter()
                .map(|item| T::json_deserialize(Some(item)))
                .collect(),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Array,
                found: JsonFieldType::for_json_value(v),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'src, K, V> JsonDeserialize<'src> for HashMap<K, V>
where
    K: FromStr + Eq + Hash + 'src,
    V: JsonDeserialize<'src>,
    K::Err: std::fmt::Debug,
{
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Object(obj)) => {
                let mut map = HashMap::new();
                for (key, val) in obj.iter() {
                    let parsed_key = K::from_str(key).map_err(|_| MerdeJsonError::InvalidKey)?;
                    let parsed_value = V::json_deserialize(Some(val))?;
                    map.insert(parsed_key, parsed_value);
                }
                Ok(map)
            }
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Object,
                found: JsonFieldType::for_json_value(v),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'src> JsonDeserialize<'src> for JsonValue<'src> {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(json_value) => Ok(variance::shorten_jsonvalue_lifetime(json_value.clone())),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'src> JsonDeserialize<'src> for JsonArray<'src> {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Array(arr)) => Ok(variance::shorten_jsonarray_lifetime(arr.clone())),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Array,
                found: JsonFieldType::for_json_value(v),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'src> JsonDeserialize<'src> for JsonObject<'src> {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
    ) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Object(obj)) => Ok(variance::shorten_jsonobject_lifetime(obj.clone())),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Object,
                found: JsonFieldType::for_json_value(v),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

/// Provides various methods useful when implementing `JsonDeserialize`.
pub trait JsonValueExt<'src, 'val>
where
    'src: 'val,
    Self: 'src,
{
    /// Coerce to `JsonObject`, returns `MerdeJsonError::MismatchedType` if not an object.
    fn as_object(&'val self) -> Result<&'val JsonObject<'src>, MerdeJsonError>;

    /// Coerce to `JsonArray`, returns `MerdeJsonError::MismatchedType` if not an array.
    fn as_array(&'val self) -> Result<&'val JsonArray<'src>, MerdeJsonError>;

    /// Coerce to `Cow<'src, str>`, returns `MerdeJsonError::MismatchedType` if not a string.
    fn as_cow_str(&'val self) -> Result<&'val Cow<'src, str>, MerdeJsonError>;

    /// Coerce to `i64`, returns `MerdeJsonError::MismatchedType` if not an integer.
    fn as_i64(&'val self) -> Result<i64, MerdeJsonError>;
}

impl<'src, 'val> JsonValueExt<'src, 'val> for JsonValue<'src>
where
    'src: 'val,
    Self: 'src,
{
    #[inline(always)]
    fn as_object(&'val self) -> Result<&'val JsonObject<'src>, MerdeJsonError> {
        match self {
            JsonValue::Object(obj) => Ok(obj),
            _ => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Object,
                found: JsonFieldType::for_json_value(self),
            }),
        }
    }

    #[inline(always)]
    fn as_array(&'val self) -> Result<&'val JsonArray<'src>, MerdeJsonError> {
        match self {
            JsonValue::Array(arr) => Ok(arr),
            _ => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Array,
                found: JsonFieldType::for_json_value(self),
            }),
        }
    }

    #[inline(always)]
    fn as_cow_str(&'val self) -> Result<&'val Cow<'src, str>, MerdeJsonError> {
        match self {
            JsonValue::Str(s) => Ok(s),
            _ => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::String,
                found: JsonFieldType::for_json_value(self),
            }),
        }
    }

    #[inline(always)]
    fn as_i64(&'val self) -> Result<i64, MerdeJsonError> {
        match self {
            JsonValue::Int(n) => Ok(*n),
            _ => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Int,
                found: JsonFieldType::for_json_value(self),
            }),
        }
    }
}

/// Writes JSON to a `Vec<u8>`. None of its methods can fail, since it doesn't target
/// an `io::Write`. You can provide your own buffer via `JsonSerializer::from_vec`.
///
/// When you're done with the serializer, you can call `JsonSerializer::into_inner` to
/// get the buffer back.
#[derive(Default)]
pub struct JsonSerializer {
    buffer: Vec<u8>,
}

impl JsonSerializer {
    /// Uses the provided buffer as the target for serialization.
    pub fn from_vec(vec: Vec<u8>) -> Self {
        JsonSerializer { buffer: vec }
    }

    /// Allocates a new buffer for serialization.
    pub fn new() -> Self {
        Self::default()
    }

    /// Writes the JSON `null` value.
    pub fn write_null(&mut self) {
        self.buffer.extend_from_slice(b"null");
    }

    /// Writes the JSON `true` or `false` value.
    pub fn write_bool(&mut self, value: bool) {
        self.buffer
            .extend_from_slice(if value { b"true" } else { b"false" });
    }

    /// Write a number as a JSON number. Numbers bigger than 2**53 might
    /// not be parsed correctly by other implementations.
    pub fn write_i64(&mut self, value: i64) {
        let _ = write!(self.buffer, "{}", value);
    }

    /// Write a floating-point number as a JSON number.
    pub fn write_f64(&mut self, value: f64) {
        let _ = write!(self.buffer, "{}", value);
    }

    /// Write a string, with escaping.
    pub fn write_str(&mut self, value: &str) {
        self.buffer.push(b'"');
        for c in value.chars() {
            match c {
                '"' => self.buffer.extend_from_slice(b"\\\""),
                '\\' => self.buffer.extend_from_slice(b"\\\\"),
                '\n' => self.buffer.extend_from_slice(b"\\n"),
                '\r' => self.buffer.extend_from_slice(b"\\r"),
                '\t' => self.buffer.extend_from_slice(b"\\t"),
                c if c.is_control() => {
                    let _ = write!(self.buffer, "\\u{:04x}", c as u32);
                }
                c => self.buffer.extend_from_slice(c.to_string().as_bytes()),
            }
        }
        self.buffer.push(b'"');
    }

    /// This writes the opening brace of an object, and gives you
    /// a guard object to write the key-value pairs. When the guard
    /// is dropped, the closing brace is written.
    pub fn write_obj(&mut self) -> ObjectGuard<'_> {
        self.buffer.push(b'{');
        ObjectGuard {
            serializer: self,
            first: true,
        }
    }

    /// This writes the opening bracket of an array, and gives you
    /// a guard object to write the elements. When the guard
    /// is dropped, the closing bracket is written.
    pub fn write_arr(&mut self) -> ArrayGuard<'_> {
        self.buffer.push(b'[');
        ArrayGuard {
            serializer: self,
            first: true,
        }
    }

    /// Get back the internal buffer
    pub fn into_inner(self) -> Vec<u8> {
        self.buffer
    }

    /// Mutably borrow the internal buffer (as a `Vec<u8>` so it's growable).
    ///
    /// This is particularly useful when you want to use an interface like format_into that expects a dyn Writer?
    pub fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }
}

/// Allows writing JSON objects
pub struct ObjectGuard<'a> {
    serializer: &'a mut JsonSerializer,
    first: bool,
}

impl<'a> ObjectGuard<'a> {
    /// Writes a key-value pair to the object.
    #[inline]
    pub fn pair(&mut self, key: &str, value: &dyn JsonSerialize) -> &mut Self {
        if !self.first {
            self.serializer.buffer.push(b',');
        }
        self.first = false;
        self.serializer.write_str(key);
        self.serializer.buffer.push(b':');
        value.json_serialize(self.serializer);
        self
    }
}

impl<'a> Drop for ObjectGuard<'a> {
    #[inline]
    fn drop(&mut self) {
        self.serializer.buffer.push(b'}');
    }
}

/// A guard object for writing an array.
pub struct ArrayGuard<'a> {
    serializer: &'a mut JsonSerializer,
    first: bool,
}

impl<'a> ArrayGuard<'a> {
    /// Writes an element to the array.
    #[inline]
    pub fn elem(&mut self, value: &dyn JsonSerialize) -> &mut Self {
        if !self.first {
            self.serializer.buffer.push(b',');
        }
        self.first = false;
        value.json_serialize(self.serializer);
        self
    }
}

impl<'a> Drop for ArrayGuard<'a> {
    #[inline]
    fn drop(&mut self) {
        self.serializer.buffer.push(b']');
    }
}

/// Implemented by anything that can be serialized to JSON.
///
/// Default implementations are provided for primitive types, strings, arrays,
/// HashMap, Option, and slices of tuples (for when you don't _need_ the
/// "hash" part of the HashMap).
///
/// `u64` and `i64` numbers, even those bigger than 2**53, are written as numbers, not strings,
/// which might trip up other JSON parsers. If that's a concern, consider writing numbers
/// as strings yourself, or sticking to `u32`.
///
/// Empty maps and vectors are written as `{}` and `[]`, respectively, not omitted.
///
/// `None` Options are omitted, not written as `null`. There is no way to specify a
/// struct field that serializes to `null` at the moment (a custom implementation could
/// use `JsonValue::Null` internally).
pub trait JsonSerialize {
    /// Write self to a `JsonSerializer`.
    fn json_serialize(&self, s: &mut JsonSerializer);

    /// Allocate a new `Vec<u8>` and serialize self to it.
    fn to_json_bytes(&self) -> Vec<u8> {
        let mut s = JsonSerializer::new();
        self.json_serialize(&mut s);
        s.into_inner()
    }

    /// Serialize self to a `String`.
    fn to_json_string(&self) -> String {
        // SAFETY: This is safe because we know that the JSON serialization
        // produced by `to_json_bytes` will always be valid UTF-8.
        unsafe { String::from_utf8_unchecked(self.to_json_bytes()) }
    }
}

impl JsonSerialize for JsonValue<'_> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        match self {
            JsonValue::Null => serializer.write_null(),
            JsonValue::Bool(b) => serializer.write_bool(*b),
            JsonValue::Int(i) => serializer.write_i64(*i),
            JsonValue::BigInt(bi) => serializer.write_str(&bi.to_string()),
            JsonValue::Float(f) => serializer.write_f64(*f),
            JsonValue::Str(s) => serializer.write_str(s),
            JsonValue::Array(arr) => arr.json_serialize(serializer),
            JsonValue::Object(obj) => obj.json_serialize(serializer),
        }
    }
}

impl JsonSerialize for JsonObject<'_> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        let mut guard = serializer.write_obj();
        for (key, value) in self.iter() {
            guard.pair(key, value);
        }
    }
}

impl JsonSerialize for JsonArray<'_> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        let mut guard = serializer.write_arr();
        for value in self.iter() {
            guard.elem(value);
        }
    }
}

impl<T> JsonSerialize for &T
where
    T: ?Sized + JsonSerialize,
{
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        let this: &T = self;
        JsonSerialize::json_serialize(this, serializer)
    }
}

impl JsonSerialize for String {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_str(self)
    }
}

impl<'a> JsonSerialize for &'a str {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_str(self)
    }
}

impl<'a> JsonSerialize for Cow<'a, str> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_str(self)
    }
}

impl JsonSerialize for u8 {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_i64(*self as i64);
    }
}

impl JsonSerialize for u16 {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_i64(*self as i64);
    }
}

impl JsonSerialize for u32 {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_i64(*self as i64);
    }
}

impl JsonSerialize for u64 {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_i64(*self as i64);
    }
}

impl JsonSerialize for i8 {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_i64(*self as i64);
    }
}

impl JsonSerialize for i16 {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_i64(*self as i64);
    }
}

impl JsonSerialize for i32 {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_i64(*self as i64);
    }
}

impl JsonSerialize for i64 {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_i64(*self);
    }
}

impl JsonSerialize for usize {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_i64(*self as i64);
    }
}

impl JsonSerialize for isize {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_i64(*self as i64);
    }
}

impl JsonSerialize for bool {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer.write_bool(*self);
    }
}

impl<K: AsRef<str>, V: JsonSerialize> JsonSerialize for HashMap<K, V> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        let mut guard = serializer.write_obj();
        for (key, value) in self {
            guard.pair(key.as_ref(), value);
        }
    }
}

impl<T: JsonSerialize> JsonSerialize for Vec<T> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        let mut guard = serializer.write_arr();
        for value in self {
            guard.elem(value);
        }
    }
}

impl<T: JsonSerialize> JsonSerialize for Option<T> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        match self {
            Some(value) => value.json_serialize(serializer),
            None => serializer.write_null(),
        }
    }
}

impl<T: JsonSerialize> JsonSerialize for &[T] {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        let mut guard = serializer.write_arr();
        for value in *self {
            guard.elem(value);
        }
    }
}

impl<V: JsonSerialize> JsonSerialize for &[(&str, V)] {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        let mut guard = serializer.write_obj();
        for (key, value) in *self {
            guard.pair(key.as_ref(), value);
        }
    }
}

/// Extension trait to provide `must_get` on `JsonObject<'_>`
pub trait JsonObjectExt<'src, T>
where
    T: JsonDeserialize<'src>,
{
    /// Gets a value from the object, returning an error if the key is missing.
    ///
    /// Because this method knows the key name, it transforms [MerdeJsonError::MissingValue] into [MerdeJsonError::MissingProperty].
    ///
    /// It does not by itself throw an error if `self.get()` returns `None`, to allow
    /// for optional fields (via the [JsonDeserialize] implementation on the [Option] type).
    fn must_get(&self, key: impl Into<Cow<'static, str>>) -> Result<T, MerdeJsonError>;
}

impl<'src, T> JsonObjectExt<'src, T> for JsonObject<'src>
where
    T: JsonDeserialize<'src>,
{
    fn must_get(&self, key: impl Into<Cow<'static, str>>) -> Result<T, MerdeJsonError> {
        let key = key.into();
        T::json_deserialize(self.get(&key)).map_err(|e| match e {
            MerdeJsonError::MissingValue => MerdeJsonError::MissingProperty(key),
            _ => e,
        })
    }
}

/// Extension trait to provide `must_get` on `JsonArray<'_>`
pub trait JsonArrayExt<'src, T>
where
    T: JsonDeserialize<'src>,
{
    /// Gets a value from the array, returning an error if the index is out of bounds.
    ///
    /// Because this method knows the index, it transforms [MerdeJsonError::MissingValue] into [MerdeJsonError::IndexOutOfBounds].
    ///
    /// It does not by itself throw an error if `self.get()` returns `None`, to allow
    /// for optional fields (via the [JsonDeserialize] implementation on the [Option] type).
    fn must_get(&self, index: usize) -> Result<T, MerdeJsonError>;
}

impl<'src, T> JsonArrayExt<'src, T> for JsonArray<'src>
where
    T: JsonDeserialize<'src>,
{
    fn must_get(&self, index: usize) -> Result<T, MerdeJsonError> {
        T::json_deserialize(self.get(index)).map_err(|e| match e {
            MerdeJsonError::MissingValue => MerdeJsonError::IndexOutOfBounds {
                index,
                len: self.len(),
            },
            _ => e,
        })
    }
}

/// Interpret a `&JsonValue` as an instance of type `T`. This may involve
/// more cloning than [from_value].
pub fn from_value_ref<'src, T>(value: &JsonValue<'src>) -> Result<T, MerdeJsonError>
where
    T: JsonDeserialize<'src>,
{
    T::json_deserialize(Some(value))
}

/// Interpret a `JsonValue` as an instance of type `T`.
pub fn from_value<'src, T>(value: JsonValue<'src>) -> Result<T, MerdeJsonError>
where
    T: JsonDeserialize<'src>,
{
    T::json_deserialize_taking_ownership(Some(value))
}

/// Deserialize an instance of type `T` from bytes of JSON text.
pub fn from_slice<'src, T>(data: &'src [u8]) -> Result<T, MerdeJsonError>
where
    T: JsonDeserialize<'src>,
{
    from_value(jiter::JsonValue::parse(data, false)?)
}

/// Deserialize an instance of type `T` from a string of JSON text.
pub fn from_str<'src, T>(s: &'src str) -> Result<T, MerdeJsonError>
where
    T: JsonDeserialize<'src>,
{
    from_slice(s.as_bytes())
}

/// Serialize the given data structure as a String of JSON.
pub fn to_string<T: JsonSerialize>(value: &T) -> String {
    value.to_json_string()
}

/// Serialize the given data structure as a JSON byte vector.
pub fn to_vec<T: JsonSerialize>(value: &T) -> Vec<u8> {
    value.to_json_bytes()
}

/// Serialize the given data structure as JSON into the I/O stream.
pub fn to_writer<W, T>(mut writer: impl std::io::Write, value: &T) -> std::io::Result<()>
where
    T: JsonSerialize,
{
    let bytes = value.to_json_bytes();
    writer.write_all(&bytes)
}

/// Allow turning a value into an "owned" variant, which can then be
/// returned, moved, etc.
///
/// This usually involves allocating buffers for `Cow<'a, str>`, etc.
pub trait ToStatic {
    /// The "owned" variant of the type. For `Cow<'a, str>`, this is `Cow<'static, str>`, for example.
    type Output: 'static;

    /// Turns the value into an "owned" variant, which can then be returned, moved, etc.
    ///
    /// This allocates, for all but the most trivial types.
    fn to_static(&self) -> Self::Output;
}

impl<'a, T> ToStatic for Cow<'a, T>
where
    T: ToOwned + ?Sized + 'static,
{
    type Output = Cow<'static, T>;

    fn to_static(&self) -> Self::Output {
        match self.clone() {
            Cow::Borrowed(b) => Cow::Owned(b.to_owned()),
            Cow::Owned(o) => Cow::Owned(o),
        }
    }
}

impl ToStatic for u8 {
    type Output = u8;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for u16 {
    type Output = u16;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for u32 {
    type Output = u32;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for u64 {
    type Output = u64;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for i8 {
    type Output = i8;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for i16 {
    type Output = i16;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for i32 {
    type Output = i32;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for i64 {
    type Output = i64;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for usize {
    type Output = usize;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for isize {
    type Output = isize;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for bool {
    type Output = bool;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for String {
    type Output = String;

    fn to_static(&self) -> Self::Output {
        self.clone()
    }
}

impl<T: ToStatic> ToStatic for Option<T> {
    type Output = Option<T::Output>;

    fn to_static(&self) -> Self::Output {
        self.as_ref().map(|v| v.to_static())
    }
}

impl<T: ToStatic> ToStatic for Vec<T> {
    type Output = Vec<T::Output>;

    fn to_static(&self) -> Self::Output {
        self.iter().map(|v| v.to_static()).collect()
    }
}

impl<K, V> ToStatic for HashMap<K, V>
where
    K: ToStatic + Eq + Hash,
    V: ToStatic,
    K::Output: Eq + Hash,
{
    type Output = HashMap<K::Output, V::Output>;

    fn to_static(&self) -> Self::Output {
        self.iter()
            .map(|(k, v)| (k.to_static(), v.to_static()))
            .collect()
    }
}

use std::collections::{HashSet, VecDeque};

impl<T: ToStatic> ToStatic for HashSet<T>
where
    T::Output: Eq + Hash,
{
    type Output = HashSet<T::Output>;

    fn to_static(&self) -> Self::Output {
        self.iter().map(|v| v.to_static()).collect()
    }
}

impl<T: ToStatic> ToStatic for VecDeque<T> {
    type Output = VecDeque<T::Output>;

    fn to_static(&self) -> Self::Output {
        self.iter().map(|v| v.to_static()).collect()
    }
}

// -------------------------------------------------------------------------
// Macros
// -------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! impl_json_deserialize {
    ($struct_name:ident { $($field:ident),+ }) => {
        impl<'src> $crate::JsonDeserialize<'src> for $struct_name<'src>
        {
            fn json_deserialize<'val>(
                value: Option<&'val $crate::JsonValue<'src>>,
            ) -> Result<Self, $crate::MerdeJsonError> {
                #[allow(unused_imports)]
                use $crate::{JsonObjectExt, JsonValueExt, MerdeJsonError};

                let obj = value.ok_or(MerdeJsonError::MissingValue)?.as_object()?;
                Ok($struct_name {
                    _boo: Default::default(),
                    $($field: obj.must_get(stringify!($field))?,)+
                })
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_json_serialize {
    ($struct_name:ident { $($field:ident),+ }) => {
        impl<'src> $crate::JsonSerialize for $struct_name<'src> {
            fn json_serialize(&self, serializer: &mut $crate::JsonSerializer) {
                #[allow(unused_imports)]
                use $crate::{JsonObjectExt, JsonValueExt, MerdeJsonError};

                let mut guard = serializer.write_obj();
                $(
                    guard.pair(stringify!($field), &self.$field);
                )+
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_to_static {
    ($struct_name:ident { $($field:ident),+ }) => {
        impl<'src> $crate::ToStatic for $struct_name<'src> {
            type Output = $struct_name<'static>;

            fn to_static(&self) -> Self::Output {
                #[allow(unused_imports)]
                use $crate::ToStatic;

                $struct_name {
                    _boo: Default::default(),
                    $($field: self.$field.to_static(),)+
                }
            }
        }
    };
}

/// Derives the specified traits for a struct.
///
/// This macro can be used to automatically implement `JsonSerialize` and `JsonDeserialize`
/// traits for a given struct. It expands to call the appropriate implementation macros
/// based on the traits specified.
///
/// # Usage
///
/// ```rust
/// use merde_json::{Fantome, JsonSerialize, JsonDeserialize};
/// use std::borrow::Cow;
///
/// #[derive(Debug, PartialEq)]
/// struct MyStruct<'src, 'val> {
///     _boo: Fantome<'src, 'val>,
///
///     field1: Cow<'val, str>,
///     field2: i32,
///     field3: bool,
/// }
///
/// merde_json::derive! {
///     impl(JsonSerialize, JsonDeserialize, ToStatic) for MyStruct {
///         field1,
///         field2,
///         field3
///     }
/// }
/// ```
///
/// This generates all three impls, but you can omit the ones you don't need.
///
/// The struct must have exactly one lifetime parameter. Additionally, even if there are no
/// borrowed fields, the struct must include a `_phantom` field of type `PhantomData<&'a ()>`,
/// where `'a` is the lifetime parameter.
///
/// Implementing other variants (no lifetimes, multiple lifetimes, etc.) with declarative macros
/// would be too complicated. At this point we'd want a whole parser / compiler / code generator
/// for this — or a proc macro, see [serde](https://serde.rs/)'s serde_derive.
#[macro_export]
macro_rules! derive {
    (impl($($trait:ident),+) for $struct_name:ident { $($field:ident),+ }) => {
        $crate::derive!(@step1 { $($trait),+ } $struct_name { $($field),+ });
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } $struct_name:ident $fields:tt) => {
        $crate::impl_trait!(@impl $trait, $struct_name $fields);
        $crate::derive!(@step1 { $($rest_traits),* } $struct_name $fields);
    };
    (@step1 { $trait:ident } $struct_name:ident $fields:tt) => {
        $crate::impl_trait!(@impl $trait, $struct_name $fields);
    };
    (@step1 { } $struct_name:ident $fields:tt) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_trait {
    (@impl JsonSerialize, $struct_name:ident { $($field:ident),+ }) => {
        $crate::impl_json_serialize!($struct_name { $($field),+ });
    };
    (@impl JsonDeserialize, $struct_name:ident { $($field:ident),+ }) => {
        $crate::impl_json_deserialize!($struct_name { $($field),+ });
    };
    (@impl ToStatic, $struct_name:ident { $($field:ident),+ }) => {
        $crate::impl_to_static!($struct_name { $($field),+ });
    };
}

/// A type you can use instead of `PhantomData` for convenience.
///
/// Note: if you're conditionally deriving `JsonSerialize` and `JsonDeserialize` for a type,
/// and you don't want the `merde_json` dependency  when it's not used, you can use
/// `merde_json_types::Fantome` instead — the derive macros will be happy with that.
///
/// This type is really just a convenience so you have less to type.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fantome<'src> {
    _boo: std::marker::PhantomData<&'src ()>,
}

impl std::fmt::Debug for Fantome<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Boo!")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_large_number() {
        // TODO: only run that when bigint is enabled
        let large_number = 4611686018427387904u64; // 2^62
        let serialized = large_number.to_json_string();
        let deserialized: u64 = from_str(&serialized).unwrap();
        assert_eq!(large_number, deserialized);
    }

    #[test]
    fn test_complex_structs() {
        use std::borrow::Cow;
        use std::collections::HashMap;

        #[derive(Debug, PartialEq)]
        struct SecondStruct<'src> {
            _boo: Fantome<'src>,

            string_field: Cow<'src, str>,
            int_field: i32,
        }

        derive! {
            impl(JsonSerialize, JsonDeserialize) for SecondStruct {
                string_field,
                int_field
            }
        }

        #[derive(Debug, PartialEq)]
        struct ComplexStruct<'src> {
            _boo: Fantome<'src>,

            string_field: Cow<'src, str>,
            u8_field: u8,
            u16_field: u16,
            u32_field: u32,
            u64_field: u64,
            i8_field: i8,
            i16_field: i16,
            i32_field: i32,
            i64_field: i64,
            usize_field: usize,
            bool_field: bool,
            option_field: Option<i32>,
            vec_field: Vec<i32>,
            hashmap_field: HashMap<String, i32>,
            second_struct_field: SecondStruct<'src>,
        }

        derive! {
            impl(JsonSerialize, JsonDeserialize) for ComplexStruct {
                string_field,
                u8_field,
                u16_field,
                u32_field,
                u64_field,
                i8_field,
                i16_field,
                i32_field,
                i64_field,
                usize_field,
                bool_field,
                option_field,
                vec_field,
                hashmap_field,
                second_struct_field
            }
        }

        let mut hashmap = HashMap::new();
        hashmap.insert("key".to_string(), 42);

        let original = ComplexStruct {
            string_field: Cow::Borrowed("test string"),
            u8_field: 8,
            u16_field: 16,
            u32_field: 32,
            u64_field: 64,
            i8_field: -8,
            i16_field: -16,
            i32_field: -32,
            i64_field: -64,
            usize_field: 100,
            bool_field: true,
            option_field: Some(42),
            vec_field: vec![1, 2, 3],
            hashmap_field: hashmap,
            second_struct_field: SecondStruct {
                _boo: Default::default(),

                string_field: Cow::Borrowed("nested string"),
                int_field: 100,
            },
            _boo: Default::default(),
        };

        let serialized = original.to_json_string();
        let deserialized: ComplexStruct = from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}
