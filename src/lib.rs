#![deny(missing_docs)]

//! `merde_json` adds convenient extension traits to the [jiter](https://crates.io/crates/jiter) crate,
//! along with a few declarative macros that allow "deriving" traits like [`JsonSerialize`], [`JsonDeserialize`],
//! and [`ToStatic`], covering the 90% use case for manipulating JSON in Rust.
//!
//! It optimizes for low compile-times and avoiding copies (but not all allocations). It's well-suited
//! for use in web servers, if you're willing to give up some of the convenience of [serde](https://crates.io/crates/serde).
//!
//! The underlying JSON parser is [jiter](https://crates.io/crates/jiter), which provides an event-based interface
//! you can choose to use when merde_json's performance simply isn't enough.
//!
//! ## Conventions + migrating from `serde_json`
//!
//! [serde](https://crates.io/crates/serde) lets you derive `Serialize` and `Deserialize` traits using
//! a proc macro:
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct MyStruct {
//!     name: String,
//!     age: u8,
//! }
//! ```
//!
//! By contrast, merde_json provides declarative macros:
//!
//! ```rust
//! use std::borrow::Cow;
//! use std::marker::PhantomData;
//!
//! #[derive(Debug, PartialEq)]
//! struct MyStruct<'a> {
//!     name: Cow<'a, str>,
//!     age: u8,
//!     _phantom: PhantomData<&'a ()>,
//! }
//!
//! merde_json::derive! {
//!     impl(JsonSerialize, JsonDeserialize) for MyStruct {
//!         name,
//!         age
//!     }
//! }
//! ```
//!
//! Declarative macros = less work to do at compile-time, as long as we follow a couple rules:
//!
//!  * All structs have a lifetime parameter
//!  * All structs have a `_phantom` field, for structs that don't use their lifetime parameter
//!  * Field names are listed twice: in the struct and in the macro (limitation of declarative macros)
//!  * Use `Cow<'a, str>` for everything, instead of choosing between `&str` and `String` on a case-by-case basis
//!
//! Read [The Secret Life Of Cows](https://deterministic.space/secret-life-of-cows.html) for a good introduction to Rust's "Copy-on-Write" types.
//!
//! ## Deserializing
//!
//! [from_str][] is a thin wrapper above jiter's API, the underlying JSON parser.
//! It gives you a `JsonValue`, which you can then destructure into a Rust value
//! via the [JsonDeserialize] trait:
//!
//! ```rust
//! # use merde_json::{JsonDeserialize, JsonSerialize, ToRustValue};
//! # use std::{borrow::Cow, marker::PhantomData};
//! #
//! # #[derive(Debug, PartialEq)]
//! # struct MyStruct<'a> {
//! #     name: Cow<'a, str>,
//! #     age: u8,
//! #     _phantom: PhantomData<&'a ()>,
//! # }
//! #
//! # merde_json::derive! {
//! #     impl(JsonSerialize, JsonDeserialize) for MyStruct { name, age }
//! # }
//! #
//! # fn main() -> Result<(), merde_json::MerdeJsonError> {
//! let input = String::from(r#"{"name": "John Doe", "age": 30}"#);
//! let value = merde_json::from_str(&input)?;
//! let my_struct = MyStruct::json_deserialize(Some(&value));
//! println!("{:?}", my_struct);
//! # Ok(())
//! # }
//! ```
//!
//! For convenience, you can use [ToRustValue::to_rust_value]:
//!
//! ```rust
//! # use merde_json::{JsonDeserialize, JsonSerialize, ToRustValue};
//! # use std::{borrow::Cow, marker::PhantomData};
//! #
//! # #[derive(Debug, PartialEq)]
//! # struct MyStruct<'a> {
//! #     name: Cow<'a, str>,
//! #     age: u8,
//! #     _phantom: PhantomData<&'a ()>,
//! # }
//! #
//! # merde_json::derive! {
//! #     impl(JsonSerialize, JsonDeserialize) for MyStruct { name, age }
//! # }
//! #
//! # fn main() -> Result<(), merde_json::MerdeJsonError> {
//! let input = String::from(r#"{"name": "John Doe", "age": 30}"#);
//! let value = merde_json::from_str(&input)?;
//! // Note: you have to specify the binding's type here.
//! // We can't use a turbofish anymore than we can with `Into::into`.
//! let my_struct: MyStruct = value.to_rust_value()?;
//! println!("{:?}", my_struct);
//! # Ok(())
//! # }
//! ```
//!
//! However, don't lose sight of the fact that `my_struct` borrows from `value`, which borrows from `input`.
//!
//! We _need_ three explicit bindings, as tempting as it would be to try and
//! inline one of them. This fails to compile with a "temporary value dropped while borrowed" error:
//!
//! ```compile_fail
//! # use merde_json::{JsonDeserialize, JsonSerialize, ToRustValue};
//! # use std::{borrow::Cow, marker::PhantomData};
//! #
//! # #[derive(Debug, PartialEq)]
//! # struct MyStruct<'a> {
//! #     name: Cow<'a, str>,
//! #     age: u8,
//! #     _phantom: PhantomData<&'a ()>,
//! # }
//! #
//! # merde_json::derive! {
//! #     impl(JsonSerialize, JsonDeserialize) for MyStruct { name, age }
//! # }
//! #
//! # fn main() -> Result<(), merde_json::MerdeJsonError> {
//! let input = String::from(r#"{"name": "John Doe", "age": 30}"#);
//! let value = merde_json::from_str(&input).unwrap();
//! let my_struct = MyStruct::json_deserialize(Some(&merde_json::from_str(&input).unwrap()));
//! println!("{:?}", my_struct);
//! # Ok(())
//! # }
//! ```
//!
//! ## Moving deserialized values around
//!
//! If you try to return a freshly-deserialized value, the compiler will stop you:
//!
//! ```compile_fail
//! # use merde_json::{JsonDeserialize, JsonSerialize, ToRustValue};
//! # use std::{borrow::Cow, marker::PhantomData};
//! #
//! # #[derive(Debug, PartialEq)]
//! # struct MyStruct<'a> {
//! #     name: Cow<'a, str>,
//! #     age: u8,
//! #     _phantom: PhantomData<&'a ()>,
//! # }
//! #
//! # merde_json::derive! {
//! #     impl(JsonSerialize, JsonDeserialize) for MyStruct { name, age }
//! # }
//! #
//! fn return_my_struct() -> MyStruct {
//!     let input = String::from(r#"{"name": "John Doe", "age": 30}"#);
//!     let value = merde_json::from_str(&input).unwrap();
//!     let my_struct: MyStruct = value.to_rust_value().unwrap();
//!     my_struct
//! }
//! # fn main() -> Result<(), merde_json::MerdeJsonError> {
//! let my_struct = return_my_struct();
//! println!("{:?}", my_struct);
//! # Ok(())
//! # }
//! ```
//!
//! This fails with:
//!
//! ```text
//! error[E0515]: cannot return value referencing local variable `input`
//!   --> src/lib.rs:163:5
//!    |
//! 20 |     let value = merde_json::from_str(&input).unwrap();
//!    |                                      ------ `input` is borrowed here
//! 21 |     let my_struct: MyStruct = value.to_rust_value().unwrap();
//! 22 |     my_struct
//!    |     ^^^^^^^^^ returns a value referencing data owned by the current function
//! ```
//!
//! And some may be thinking "self-referential types!", but I'm thinking "heap allocations":
//!
//! ```rust
//! # use merde_json::{JsonDeserialize, JsonSerialize, ToRustValue, ToStatic};
//! # use std::{borrow::Cow, marker::PhantomData};
//! #
//! # #[derive(Debug, PartialEq)]
//! # struct MyStruct<'a> {
//! #     name: Cow<'a, str>,
//! #     age: u8,
//! #     _phantom: PhantomData<&'a ()>,
//! # }
//! #
//! # merde_json::derive! {
//! #     impl(JsonSerialize, JsonDeserialize, ToStatic) for MyStruct { name, age }
//! # }
//! #
//! fn return_my_struct() -> MyStruct<'static> {
//!     let input = String::from(r#"{"name": "John Doe", "age": 30}"#);
//!     let value = merde_json::from_str(&input).unwrap();
//!     let my_struct: MyStruct = value.to_rust_value().unwrap();
//!     //           👇
//!     my_struct.to_static()
//! }
//! # fn main() -> Result<(), merde_json::MerdeJsonError> {
//! let my_struct = return_my_struct();
//! println!("{:?}", my_struct);
//! # Ok(())
//! # }
//! ```
//!
//! [ToStatic::to_static] lets you go from `MyStruct<'a>` to `MyStruct<'static>`. A default implementation
//! is provided for on `Cow<'a, T>` and a bunch of other primitive types. It can be derived via the [derive!]
//! macro.
//!
//! ## Deserializing mixed-type arrays
//!
//! Real-world JSON payloads have arrays with mixed types. You can keep them as [Vec] of [JsonValue]
//! until you know what to do with them:
//!
//! ```rust
//! use merde_json::{JsonDeserialize, JsonSerialize, ToRustValue, JsonValue, MerdeJsonError};
//! use std::{borrow::Cow, marker::PhantomData};
//!
//! #[derive(Debug, PartialEq)]
//! struct MixedArray<'a, 'inner> {
//!     items: JsonValue<'inner>,
//!     _phantom: PhantomData<&'a ()>,
//! }
//!
//! merde_json::derive! { impl(JsonDeserialize) for MixedArray { items } }
//!
//! fn main() -> Result<(), merde_json::MerdeJsonError> {
//!     let input = r#"[1, "two", true, null, {"key": "value"}]"#;
//!     let value = merde_json::from_str(input)?;
//!     let mixed_array: MixedArray = value.to_rust_value()?;
//!
//!     println!("Mixed array: {:?}", mixed_array);
//!
//!     // You can then process each item based on its type
//!     for (index, item) in mixed_array.items.as_array()?.iter().enumerate() {
//!         match item {
//!             JsonValue::Int(i) => println!("Item {} is an integer: {}", index, i),
//!             JsonValue::Str(s) => println!("Item {} is a string: {}", index, s),
//!             JsonValue::Bool(b) => println!("Item {} is a boolean: {}", index, b),
//!             JsonValue::Null => println!("Item {} is null", index),
//!             JsonValue::Object(obj) => println!("Item {} is an object: {:?}", index, obj),
//!             _ => println!("Item {} is of another type", index),
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Serializing
//!
//! Serializing typically looks like:
//!
//! ```rust
//! # use merde_json::{JsonSerialize, JsonDeserialize, ToRustValue};
//! # use std::{borrow::Cow, marker::PhantomData};
//! #
//! # #[derive(Debug, PartialEq)]
//! # struct MyStruct<'a> {
//! #     name: Cow<'a, str>,
//! #     age: u8,
//! #     _phantom: PhantomData<&'a ()>,
//! # }
//! #
//! # merde_json::derive! {
//! #     impl(JsonSerialize, JsonDeserialize) for MyStruct { name, age }
//! # }
//! #
//! # fn main() -> Result<(), merde_json::MerdeJsonError> {
//! let original = MyStruct {
//!     name: Cow::Borrowed("John Doe"),
//!     age: 30,
//!     _phantom: PhantomData
//! };
//!
//! let serialized = original.to_json_string();
//! println!("{}", serialized);
//!
//! let ms = merde_json::from_str(&serialized)?;
//! let ms: MyStruct = ms.to_rust_value()?;
//! assert_eq!(original, ms);
//! # Ok(())
//! # }
//! ```
//!
//! ## Reducing allocations when serializing
//!
//! If you want more control over the buffer, for example you'd like to re-use the same
//! `Vec<u8>` for multiple serializations, you can use [JsonSerializer::from_vec]:
//!
//! ```rust
//! # use merde_json::{JsonSerialize, JsonDeserialize, ToRustValue};
//! # use std::{borrow::Cow, marker::PhantomData};
//! #
//! # #[derive(Debug, PartialEq)]
//! # struct MyStruct<'a> {
//! #     name: Cow<'a, str>,
//! #     age: u8,
//! #     _phantom: PhantomData<&'a ()>,
//! # }
//! #
//! # merde_json::derive! {
//! #     impl(JsonSerialize, JsonDeserialize) for MyStruct { name, age }
//! # }
//! #
//! # fn main() -> Result<(), merde_json::MerdeJsonError> {
//! let original = MyStruct {
//!     name: Cow::Borrowed("John Doe"),
//!     age: 30,
//!     _phantom: PhantomData
//! };
//!
//! let mut buffer = Vec::new();
//! for _ in 0..3 {
//!     buffer.clear();
//!     let mut serializer = merde_json::JsonSerializer::from_vec(buffer);
//!     original.json_serialize(&mut serializer);
//!     buffer = serializer.into_inner();
//!
//!     let ms = merde_json::from_slice(&buffer)?;
//!     let ms = ms.to_rust_value()?;
//!     assert_eq!(original, ms);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Note that serialization is infallible, since it targest a memory buffer rather than
//! a Writer, and we assume allocations cannot fail (like most Rust code out there currently).
//!
//! Keeping in mind that a `Vec` that grows doesn't give its memory back unless you ask for it
//! explicitly via [Vec::shrink_to_fit] or [Vec::shrink_to], for example.
//!
//! ## Caveats & limitations
//!
//! Most of this crate is extremely naive, on purpose.
//!
//! For example, deep data structures _will_ blow up the stack, since deserialization is recursive.
//!
//! Deserialization round-trips through [jiter::JsonValue], which contains types like [std::sync::Arc],
//! small vecs, lazy hash maps, etc. — building them simply to destructure from them is a waste of CPU
//! cycles, and if it shows up in your profiles, it's time to move on to jiter's event-based parser,
//! [jiter::Jiter].
//!
//! If you expect an `u32` but the JSON payload has a floating-point number, it'll get rounded.
//!
//! If you expect a `u32` but the JSON payload is greater than `u32::MAX`, you'll get a
//! [MerdeJsonError::OutOfRange] error.
//!
//! There's no control over allowing Infinity/NaN in JSON numbers: you can work around that
//! by calling [jiter::JsonValue::parse] yourself.
//!
//! Serialization can't be pretty: it never produces unnecessary spaces, newlines, etc.
//! If your performance characteristics allow it, you may look into [formatjson](https://crates.io/crates/formatjson)
//!
//! Serialization may produce JSON payloads that other parsers will reject or parse incorrectly,
//! specifically for numbers above 2^53 or below -2^53.
//!
//! There is no built-in facility for serializing/deserializing strings from numbers.
//!
//! If `merde_json` doesn't work for you, it's very likely that your use case is not supported, and
//! you should look at [serde](https://crates.io/crates/serde) instead.
//!
//! ## FAQ
//!
//! ### What's with the `Option` in the `JsonDeserialize` interface?
//!
//! This allows `Option<T>` to ignore missing values. All other implementations should
//! return `MerdeJsonError::MissingValue` if the option is `None` — this is later turned
//! into `MerdeJsonError::MissingProperty` with the field name./
//!
//! ### What do I do about `#[serde(rename_all = "camelCase")]`?
//!
//! Make your actual struct fields `camelCase`, and slap `#[allow(non_snake_case)]` on
//! top of your struct. Sorry!
//!
//! ### What do I do about `#[serde(borrow)]`?
//!
//! That's the default and only mode — use `Cow<'a, str>` for all strings, do `.to_static()`
//! if you need to move the struct.

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
pub trait JsonDeserialize<'inner, 'borrow>
where
    Self: Sized + 'borrow,
    'inner: 'borrow,
{
    /// Destructures a JSON value into a Rust value
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError>;
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for &'borrow str
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Str(s)) => match s {
                Cow::Borrowed(s) => Ok(s),
                Cow::Owned(s) => Ok(s.as_str()),
            },
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::String,
                found: v.into(),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for Cow<'borrow, str>
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Str(s)) => Ok(s.clone()),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::String,
                found: v.into(),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for u8
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        u64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for u16
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        u64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for u32
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        u64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for u64
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Int(n)) => (*n).try_into().map_err(|_| MerdeJsonError::OutOfRange),
            Some(JsonValue::Float(f)) => Ok((*f).round() as u64),
            Some(JsonValue::BigInt(bi)) => bi.try_into().map_err(|_| MerdeJsonError::OutOfRange),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Int,
                found: v.into(),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for i8
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        i64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for i16
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        i64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for i32
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        i64::json_deserialize(value)?
            .try_into()
            .map_err(|_| MerdeJsonError::OutOfRange)
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for i64
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Int(n)) => Ok(*n),
            Some(JsonValue::Float(f)) => Ok((*f).round() as i64),
            Some(JsonValue::BigInt(bi)) => bi.try_into().map_err(|_| MerdeJsonError::OutOfRange),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Int,
                found: v.into(),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for usize
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Int(n)) => (*n).try_into().map_err(|_| MerdeJsonError::OutOfRange),
            Some(JsonValue::Float(f)) => ((*f).round() as i64)
                .try_into()
                .map_err(|_| MerdeJsonError::OutOfRange),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Int,
                found: v.into(),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for bool
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Bool(b)) => Ok(*b),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Bool,
                found: v.into(),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}
impl<'inner, 'borrow, T> JsonDeserialize<'inner, 'borrow> for Option<T>
where
    T: JsonDeserialize<'inner, 'borrow>,
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Null) => Ok(None),
            Some(v) => T::json_deserialize(Some(v)).map(Some),
            None => Ok(None),
        }
    }
}

impl<'inner, 'borrow, T> JsonDeserialize<'inner, 'borrow> for Vec<T>
where
    T: JsonDeserialize<'inner, 'borrow>,
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        match value {
            Some(JsonValue::Array(arr)) => arr
                .iter()
                .map(|item| T::json_deserialize(Some(item)))
                .collect(),
            Some(v) => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Array,
                found: v.into(),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'inner, 'borrow, K, V> JsonDeserialize<'inner, 'borrow> for HashMap<K, V>
where
    'inner: 'borrow,
    K: FromStr + Eq + Hash + 'borrow,
    V: JsonDeserialize<'inner, 'borrow>,
    K::Err: std::fmt::Debug,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
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
                found: v.into(),
            }),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for JsonValue<'inner>
where
    'inner: 'borrow,
{
    fn json_deserialize(value: Option<&'borrow JsonValue<'inner>>) -> Result<Self, MerdeJsonError> {
        match value {
            Some(json_value) => Ok(json_value.clone()),
            None => Err(MerdeJsonError::MissingValue),
        }
    }
}

/// Provides various methods useful when implementing `JsonDeserialize`.
pub trait JsonValueExt<'inner, 'borrow>
where
    'inner: 'borrow,
    Self: 'inner,
{
    /// Coerce to `JsonObject`, returns `MerdeJsonError::MismatchedType` if not an object.
    fn as_object(&'borrow self) -> Result<&'borrow JsonObject<'inner>, MerdeJsonError>;

    /// Coerce to `JsonArray`, returns `MerdeJsonError::MismatchedType` if not an array.
    fn as_array(&'borrow self) -> Result<&'borrow JsonArray<'inner>, MerdeJsonError>;

    /// Coerce to `Cow<'inner, str>`, returns `MerdeJsonError::MismatchedType` if not a string.
    fn as_cow_str(&'borrow self) -> Result<&'borrow Cow<'inner, str>, MerdeJsonError>;

    /// Coerce to `i64`, returns `MerdeJsonError::MismatchedType` if not an integer.
    fn as_i64(&'borrow self) -> Result<i64, MerdeJsonError>;
}

impl<'inner, 'borrow> JsonValueExt<'inner, 'borrow> for JsonValue<'inner>
where
    'inner: 'borrow,
    Self: 'inner,
{
    fn as_object(&'borrow self) -> Result<&'borrow JsonObject<'inner>, MerdeJsonError> {
        match self {
            JsonValue::Object(obj) => Ok(obj),
            _ => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Object,
                found: self.into(),
            }),
        }
    }

    fn as_array(&'borrow self) -> Result<&'borrow JsonArray<'inner>, MerdeJsonError> {
        match self {
            JsonValue::Array(arr) => Ok(arr),
            _ => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Array,
                found: self.into(),
            }),
        }
    }

    fn as_cow_str(&'borrow self) -> Result<&'borrow Cow<'inner, str>, MerdeJsonError> {
        match self {
            JsonValue::Str(s) => Ok(s),
            _ => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::String,
                found: self.into(),
            }),
        }
    }

    fn as_i64(&'borrow self) -> Result<i64, MerdeJsonError> {
        match self {
            JsonValue::Int(n) => Ok(*n),
            _ => Err(MerdeJsonError::MismatchedType {
                expected: JsonFieldType::Int,
                found: self.into(),
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
            JsonValue::Array(arr) => {
                let mut guard = serializer.write_arr();
                for value in arr.iter() {
                    guard.elem(value);
                }
            }
            JsonValue::Object(obj) => {
                let mut guard = serializer.write_obj();
                for (key, value) in obj.iter() {
                    guard.pair(key, value);
                }
            }
        }
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
pub trait JsonObjectExt<'borrow, 'inner, T>
where
    'inner: 'borrow,
    T: JsonDeserialize<'inner, 'borrow> + 'borrow,
{
    /// Gets a value from the object, returning an error if the key is missing.
    ///
    /// Because this method knows the key name, it transforms [MerdeJsonError::MissingValue] into [MerdeJsonError::MissingProperty].
    ///
    /// It does not by itself throw an error if `self.get()` returns `None`, to allow
    /// for optional fields (via the [JsonDeserialize] implementation on the [Option] type).
    fn must_get(&'borrow self, key: &'static str) -> Result<T, MerdeJsonError>;
}

impl<'borrow, 'inner, T> JsonObjectExt<'borrow, 'inner, T> for JsonObject<'inner>
where
    'inner: 'borrow,
    T: JsonDeserialize<'inner, 'borrow> + 'borrow,
{
    fn must_get(&'borrow self, key: &'static str) -> Result<T, MerdeJsonError> {
        T::json_deserialize(self.get(key)).map_err(|e| match e {
            MerdeJsonError::MissingValue => MerdeJsonError::MissingProperty(key),
            _ => e,
        })
    }
}

/// Parses a JSON byte string into a [JsonValue].
pub fn from_slice(data: &[u8]) -> Result<jiter::JsonValue<'_>, MerdeJsonError> {
    Ok(jiter::JsonValue::parse(data, false)?)
}

/// Parses a JSON string into a [JsonValue].
pub fn from_str(s: &str) -> Result<jiter::JsonValue<'_>, MerdeJsonError> {
    from_slice(s.as_bytes())
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

/// Extension trait to provide `to_rust_value` on `JsonValue<'_>`
///
/// Which allows you to do something like:
///
/// ```rust
/// use merde_json::{JsonDeserialize, JsonSerialize, ToRustValue};
/// use std::{borrow::Cow, marker::PhantomData};
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct MyStruct<'a> {
///     name: Cow<'a, str>,
///     age: u8,
///     _phantom: PhantomData<&'a ()>,
/// }
///
/// merde_json::derive! {
///     impl (JsonSerialize, JsonDeserialize) for MyStruct { name, age }
/// }
///
/// let input = r#"{"name": "John Doe", "age": 30}"#;
/// let value: merde_json::JsonValue = merde_json::from_str(input).unwrap();
/// let my_struct: MyStruct = value.to_rust_value().unwrap();
/// println!("{:?}", my_struct);
/// ```
pub trait ToRustValue<'inner, 'borrow, T>
where
    'inner: 'borrow,
    T: JsonDeserialize<'inner, 'borrow> + 'borrow,
{
    /// Flips the calling convention of [JsonDeserialize::json_deserialize] to turn a [JsonValue] into a Rust value.
    ///
    /// Fallible, since the `JsonValue` might not match the structure we expect.
    fn to_rust_value(&'borrow self) -> Result<T, MerdeJsonError>;
}

impl<'inner, 'borrow, T> ToRustValue<'inner, 'borrow, T> for JsonValue<'inner>
where
    'inner: 'borrow,
    T: JsonDeserialize<'inner, 'borrow> + 'borrow,
{
    fn to_rust_value(&'borrow self) -> Result<T, MerdeJsonError> {
        JsonDeserialize::json_deserialize(Some(self))
    }
}

// -------------------------------------------------------------------------
// Macros
// -------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! impl_json_deserialize {
    ($struct_name:ident { $($field:ident),+ }) => {
        impl<'inner, 'borrow> $crate::JsonDeserialize<'inner, 'borrow> for $struct_name<'inner, 'borrow>
        where
            'inner: 'borrow,
        {
            fn json_deserialize(
                value: Option<&'borrow $crate::JsonValue<'inner>>,
            ) -> Result<Self, $crate::MerdeJsonError> {
                #[allow(unused_imports)]
                use $crate::{JsonObjectExt, JsonValueExt, MerdeJsonError, ToRustValue};

                let obj = value.ok_or(MerdeJsonError::MissingValue)?.as_object()?;
                Ok($struct_name {
                    $($field: obj.must_get(stringify!($field))?,)+
                    _phantom: Default::default()
                })
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_json_serialize {
    ($struct_name:ident { $($field:ident),+ }) => {
        impl<'inner, 'borrow> $crate::JsonSerialize for $struct_name<'inner, 'borrow> {
            fn json_serialize(&self, serializer: &mut $crate::JsonSerializer) {
                #[allow(unused_imports)]
                use $crate::{JsonObjectExt, JsonValueExt, MerdeJsonError, ToRustValue};

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
        impl<'inner, 'borrow> $crate::ToStatic<$struct_name<'inner, 'borrow>> for $struct_name<'inner, 'borrow> {
            type Output = $struct_name<'static>;

            fn to_static(&self) -> Self::Output {
                #[allow(unused_imports)]
                use $crate::ToStatic;

                $struct_name {
                    $($field: self.$field.to_static(),)+
                    _phantom: std::marker::PhantomData,
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
/// use std::{borrow::Cow, marker::PhantomData};
///
/// #[derive(Debug, PartialEq)]
/// struct MyStruct<'a> {
///     field1: Cow<'a, str>,
///     field2: i32,
///     field3: bool,
///
///     _phantom: PhantomData<&'a ()>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_large_number() {
        let large_number = 4611686018427387904u64; // 2^62
        let serialized = large_number.to_json_string();
        let deserialized: u64 = from_str(&serialized).unwrap().to_rust_value().unwrap();
        assert_eq!(large_number, deserialized);
    }

    #[test]
    fn test_complex_structs() {
        use std::borrow::Cow;
        use std::collections::HashMap;
        use std::marker::PhantomData;

        #[derive(Debug, PartialEq)]
        struct SecondStruct<'inner, 'borrow> {
            string_field: Cow<'borrow, str>,
            int_field: i32,
            _phantom: PhantomData<(&'inner (), &'borrow ())>,
        }

        derive! {
            impl(JsonSerialize, JsonDeserialize) for SecondStruct {
                string_field,
                int_field
            }
        }

        #[derive(Debug, PartialEq)]
        struct ComplexStruct<'inner, 'borrow> {
            string_field: Cow<'borrow, str>,
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
            second_struct_field: SecondStruct<'inner, 'borrow>,
            _phantom: PhantomData<(&'inner (), &'borrow ())>,
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
                string_field: Cow::Borrowed("nested string"),
                int_field: 100,
                _phantom: PhantomData,
            },
            _phantom: PhantomData,
        };

        let serialized = original.to_json_string();
        let deserialized = from_str(&serialized).unwrap();
        let deserialized: ComplexStruct = deserialized.to_rust_value().unwrap();

        assert_eq!(original, deserialized);
    }
}
