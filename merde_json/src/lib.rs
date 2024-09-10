#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod parser;

use jiter::JiterError;
use merde_types::{Array, Map, MerdeError, Value, ValueDeserialize};
use parser::bytes_to_value;

use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Write;

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
            Value::Float(f) => serializer.write_f64(*f),
            Value::Str(s) => serializer.write_str(s),
            Value::Array(arr) => arr.json_serialize(serializer),
            Value::Map(map) => map.json_serialize(serializer),
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

/// Unifies [MerdeError] and [JiterError] into a single type
#[derive(Debug)]
pub enum MerdeJsonError {
    /// A [MerdeError]
    MerdeError(MerdeError),
    /// A [JiterError]
    JiterError(JiterError),
}

impl From<MerdeError> for MerdeJsonError {
    fn from(e: MerdeError) -> Self {
        MerdeJsonError::MerdeError(e)
    }
}

impl From<JiterError> for MerdeJsonError {
    fn from(e: JiterError) -> Self {
        MerdeJsonError::JiterError(e)
    }
}

/// Deserialize an instance of type `T` from bytes of JSON text.
pub fn from_slice_via_value<'s, T>(data: &'s [u8]) -> Result<T, MerdeJsonError>
where
    T: ValueDeserialize<'s>,
{
    Ok(merde_types::from_value(bytes_to_value(data)?)?)
}

/// Deserialize an instance of type `T` from a string of JSON text.
pub fn from_str_via_value<'s, T>(s: &'s str) -> Result<T, MerdeJsonError>
where
    T: ValueDeserialize<'s>,
{
    match from_slice_via_value(s.as_bytes()) {
        Ok(v) => Ok(v),
        Err(e) => match e {
            MerdeJsonError::MerdeError(me) => Err(me.into()),
            MerdeJsonError::JiterError(je) => {
                eprintln!("JSON parsing error: {:?}", je);
                let context_start = je.index.saturating_sub(20);
                let context_end = (je.index + 20).min(s.len());
                let context = &s[context_start..context_end];

                eprintln!("Error context:");
                for (i, c) in context.char_indices() {
                    if i + context_start == je.index {
                        eprint!("\x1b[48;2;255;200;200m\x1b[97m{}\x1b[0m", c);
                    } else {
                        eprint!("\x1b[48;2;200;200;255m\x1b[97m{}\x1b[0m", c);
                    }
                }
                eprintln!();

                Err(je.into())
            }
        },
    }
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
