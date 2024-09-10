#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod iterator_samples;

use merde::{Array, CowStr, Map, MerdeError, Value, ValueType};

use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;
use std::str::FromStr;

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
/// use `Value::Null` internally).
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

impl JsonSerialize for Value<'_> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        match self {
            Value::Null => serializer.write_null(),
            Value::Bool(b) => serializer.write_bool(*b),
            Value::Int(i) => serializer.write_i64(*i),
            Value::BigInt(bi) => serializer.write_str(&bi.to_string()),
            Value::Float(f) => serializer.write_f64(*f),
            Value::Str(s) => serializer.write_str(s),
            Value::Array(arr) => arr.json_serialize(serializer),
            Value::Object(obj) => obj.json_serialize(serializer),
        }
    }
}

impl JsonSerialize for Map<'_> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        let mut guard = serializer.write_obj();
        for (key, value) in self.iter() {
            guard.pair(key, value);
        }
    }
}

impl JsonSerialize for Array<'_> {
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

/// Interpret a `&Value` as an instance of type `T`. This may involve
/// more cloning than [from_value].
pub fn from_value_ref<'s, T>(value: &Value<'s>) -> Result<T, MerdeError>
where
    T: ValueDeserialize<'s>,
{
    T::json_deserialize(Some(value))
}

/// Interpret a `Value` as an instance of type `T`.
pub fn from_value<'s, T>(value: Value<'s>) -> Result<T, MerdeError>
where
    T: ValueDeserialize<'s>,
{
    T::json_deserialize_taking_ownership(Some(value))
}

/// Deserialize an instance of type `T` from bytes of JSON text.
pub fn from_slice<'s, T>(data: &'s [u8]) -> Result<T, MerdeError>
where
    T: ValueDeserialize<'s>,
{
    from_value(jiter::Value::parse(data, false)?)
}

/// Deserialize an instance of type `T` from a string of JSON text.
pub fn from_str<'s, T>(s: &'s str) -> Result<T, MerdeError>
where
    T: ValueDeserialize<'s>,
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
    ($struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        impl<$lifetime> $crate::JsonDeserialize<$lifetime> for $struct_name<$lifetime>
        {
            fn json_deserialize<'val>(
                value: Option<&'val $crate::Value<$lifetime>>,
            ) -> Result<Self, $crate::MerdeError> {
                #[allow(unused_imports)]
                use $crate::{JsonObjectExt, ValueExt, MerdeError};

                let obj = value.ok_or(MerdeError::MissingValue)?.as_object()?;
                Ok($struct_name {
                    $($field: obj.must_get(stringify!($field))?,)+
                })
            }
        }
    };

    ($struct_name:ident { $($field:ident),+ }) => {
        impl $crate::JsonDeserialize<'static> for $struct_name
        {
            fn json_deserialize<'val>(
                value: Option<&'val $crate::Value<'_>>,
            ) -> Result<Self, $crate::MerdeError> {
                #[allow(unused_imports)]
                use $crate::{JsonObjectExt, ValueExt, MerdeError};

                let obj = value.ok_or(MerdeError::MissingValue)?.as_object()?;
                Ok($struct_name {
                    $($field: obj.must_get(stringify!($field))?,)+
                })
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_json_serialize {
    ($struct_name:ident < $lifetime:lifetime > { $($field:ident),+ }) => {
        impl<$lifetime> $crate::JsonSerialize for $struct_name<$lifetime> {
            fn json_serialize(&self, serializer: &mut $crate::JsonSerializer) {
                #[allow(unused_imports)]
                use $crate::{JsonObjectExt, ValueExt, MerdeError};

                let mut guard = serializer.write_obj();
                $(
                    guard.pair(stringify!($field), &self.$field);
                )+
            }
        }
    };

    ($struct_name:ident { $($field:ident),+ }) => {
        impl $crate::JsonSerialize for $struct_name {
            fn json_serialize(&self, serializer: &mut $crate::JsonSerializer) {
                #[allow(unused_imports)]
                use $crate::{JsonObjectExt, ValueExt, MerdeError};

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
    ($struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        impl<$lifetime> $crate::ToStatic for $struct_name<$lifetime> {
            type Output = $struct_name<'static>;

            fn to_static(&self) -> Self::Output {
                #[allow(unused_imports)]
                use $crate::ToStatic;

                $struct_name {
                    $($field: self.$field.to_static(),)+
                }
            }
        }
    };

    ($struct_name:ident { $($field:ident),+ }) => {
        impl $crate::ToStatic for $struct_name {
            type Output = $struct_name;

            fn to_static(&self) -> Self::Output {
                #[allow(unused_imports)]
                use $crate::ToStatic;

                $struct_name {
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
/// use merde_json::{JsonSerialize, JsonDeserialize};
/// use std::borrow::Cow;
///
/// #[derive(Debug, PartialEq)]
/// struct MyStruct<'s> {
///     field1: Cow<'s, str>,
///     field2: i32,
///     field3: bool,
/// }
///
/// merde_json::derive! {
///     impl(JsonSerialize, JsonDeserialize, ToStatic) for MyStruct<'s> {
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
    // cow variants
    (impl($($trait:ident),+) for $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::derive!(@step1 { $($trait),+ } $struct_name <$lifetime> { $($field),+ });
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } $struct_name:ident <$lifetime:lifetime> $fields:tt) => {
        $crate::impl_trait!(@impl $trait, $struct_name <$lifetime> $fields);
        $crate::derive!(@step1 { $($rest_traits),* } $struct_name <$lifetime> $fields);
    };
    (@step1 { $trait:ident } $struct_name:ident <$lifetime:lifetime> $fields:tt) => {
        $crate::impl_trait!(@impl $trait, $struct_name <$lifetime> $fields);
    };
    (@step1 { } $struct_name:ident <$lifetime:lifetime> $fields:tt) => {};

    // owned variants
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
    // cow variants
    (@impl JsonSerialize, $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::impl_json_serialize!($struct_name <$lifetime> { $($field),+ });
    };
    (@impl JsonDeserialize, $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::impl_json_deserialize!($struct_name <$lifetime> { $($field),+ });
    };
    (@impl ToStatic, $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::impl_to_static!($struct_name <$lifetime> { $($field),+ });
    };

    // owned variants
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
        struct SecondStruct<'s> {
            string_field: Cow<'s, str>,
            int_field: i32,
        }

        derive! {
            impl(JsonSerialize, JsonDeserialize) for SecondStruct<'s> {
                string_field,
                int_field
            }
        }

        #[derive(Debug, PartialEq)]
        struct ComplexStruct<'s> {
            string_field: Cow<'s, str>,
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
            second_struct_field: SecondStruct<'s>,
        }

        derive! {
            impl(JsonSerialize, JsonDeserialize) for ComplexStruct<'s> {
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
                string_field: Cow::Borrowed("nested string"),
                int_field: 100,
            },
        };

        let serialized = original.to_json_string();
        let deserialized: ComplexStruct = from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}
