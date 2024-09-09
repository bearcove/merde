[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/merde_json.svg)](https://crates.io/crates/merde_json)
[![docs.rs](https://docs.rs/merde_json/badge.svg)](https://docs.rs/merde_json)

# merde_json

![The merde_json logo: a glorious poop floating above a pair of hands](https://github.com/user-attachments/assets/763d60e0-5101-48af-bc72-f96f516a5d0f)

_Logo by [MisiasArt](https://misiasart.carrd.co)_

`merde_json` covers the "90% use case" for JSON manipulation via traits, declarative macros, and a bit of discipline.

It optimizes for low compile-times and avoiding copies (but not all allocations). It's well-suited
for use in web servers, if you're willing to give up some of the comforts of [proc macros](https://crates.io/crates/serde).

The underlying JSON parser is [jiter](https://crates.io/crates/jiter), which provides an event-based interface
you can choose to use when merde_json's performance simply isn't enough.

## Conventions + migrating from `serde_json`

[serde](https://crates.io/crates/serde) lets you derive `Serialize` and `Deserialize` traits using
a proc macro:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MyStruct {
    name: String,
    age: u8,
}
```

By contrast, merde_json provides declarative macros:

```rust
use merde_json::Fantome;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
struct MyStruct<'s> {
    _boo: Fantome<'s>,

    name: Cow<'s, str>,
    age: u8,
}

merde_json::derive! {
    impl(JsonSerialize, JsonDeserialize) for MyStruct {
        name,
        age
    }
}
```

Declarative macros = less work to do at compile-time, as long as we follow a couple rules:

 * All structs have an `'s` lifetime parameter
 * All structs have a `_boo` field, for structs that don't use their lifetime parameter
 * Field names are listed twice: in the struct and in the macro (limitation of declarative macros)
 * Use `Cow<'val, str>` for all your strings, instead of choosing between `&str` and `String` on a case-by-case basis

Read [The Secret Life Of Cows](https://deterministic.space/secret-life-of-cows.html) for a good introduction to Rust's "Copy-on-Write" types.

## Deserializing

[from_str][] is a convenient method that parses JSON and immediately destructures it into a Rust value
via the [JsonDeserialize] trait:

```rust
# use merde_json::{Fantome, JsonDeserialize, JsonSerialize};
# use std::borrow::Cow;
#
# #[derive(Debug, PartialEq)]
# struct MyStruct<'s> {
#     _boo: Fantome<'s>,
#
#     name: Cow<'s, str>,
#     age: u8,
# }
#
# merde_json::derive! {
#     impl(JsonSerialize, JsonDeserialize) for MyStruct { name, age }
# }
#
# fn main() -> Result<(), merde_json::MerdeJsonError> {
let input = String::from(r#"{"name": "John Doe", "age": 30}"#);
let value = merde_json::from_str(&input)?;
let my_struct = MyStruct::json_deserialize(Some(&value));
println!("{:?}", my_struct);
# Ok(())
# }
```

There are other convenience methods, to deserialize from byte slices, or from
already-parsed `JsonValue`.

## Moving deserialized values around

How do you return a freshly-deserialized value, with this annoying lifetime?

Set it to `'static`! However, this fails because the deserialized value is
not `T<'static>` — it still borrows from the source.

This code fails to compile:

```compile_fail
# use merde_json::{Fantome, JsonDeserialize, JsonSerialize};
# use std::borrow::Cow;
#
# #[derive(Debug, PartialEq)]
# struct MyStruct<'s> {
#     _boo: Fantome<'s>,
#     name: Cow<'s, str>,
#     age: u8,
# }
#
# merde_json::derive! {
#     impl(JsonSerialize, JsonDeserialize) for MyStruct { name, age }
# }
#
fn return_my_struct() -> MyStruct<'static> {
    let input = String::from(r#"{"name": "John Doe", "age": 30}"#);
    merde_json::from_str(&input).unwrap()
}
# fn main() -> Result<(), merde_json::MerdeJsonError> {
let my_struct = return_my_struct();
println!("{:?}", my_struct);
# Ok(())
# }
```

...with:

```text
error[E0515]: cannot return value referencing local variable `input`
  --> merde_json/src/lib.rs:126:5
   |
20 |     merde_json::from_str(&input).unwrap()
   |     ^^^^^^^^^^^^^^^^^^^^^------^^^^^^^^^^
   |     |                    |
   |     |                    `input` is borrowed here
   |     returns a value referencing data owned by the current function
```

Deriving the [ToStatic] trait lets you go from `MyStruct<'s>` to `MyStruct<'static>`:

```rust
# use merde_json::{Fantome, JsonDeserialize, JsonSerialize, ToStatic};
# use std::borrow::Cow;
#
# #[derive(Debug, PartialEq)]
# struct MyStruct<'s> {
#     _boo: Fantome<'s>,
#     name: Cow<'s, str>,
#     age: u8,
# }
#
merde_json::derive! {
    //                                     👇
    impl(JsonSerialize, JsonDeserialize, ToStatic) for MyStruct { name, age }
}

fn return_my_struct() -> MyStruct<'static> {
    let input = String::from(r#"{"name": "John Doe", "age": 30}"#);
    let ms: MyStruct = merde_json::from_str(&input).unwrap();
    ms.to_static()
}
# fn main() -> Result<(), merde_json::MerdeJsonError> {
let my_struct = return_my_struct();
println!("{:?}", my_struct);
# Ok(())
# }
```

Of course, [ToStatic::to_static] often involves heap allocations. If you're just temporarily
processing some JSON payload, consider accepting a callback instead and passing it a shared
reference to your value — that works more often than you'd think!

## Deserializing mixed-type arrays

Real-world JSON payloads can have arrays with mixed types. You can keep them as [Vec] of [JsonValue]
until you know what to do with them:

```rust
use merde_json::{Fantome, JsonDeserialize, JsonSerialize, JsonValue, MerdeJsonError};

#[derive(Debug, PartialEq)]
struct MixedArray<'s> {
    _boo: Fantome<'s>,
    items: Vec<JsonValue<'s>>,
}

merde_json::derive! { impl(JsonDeserialize) for MixedArray { items } }

fn main() -> Result<(), merde_json::MerdeJsonError> {
    let input = r#"{
        "items": [42, "two", true]
    }"#;
    let mixed_array: MixedArray = merde_json::from_str(input)?;

    println!("Mixed array: {:?}", mixed_array);

    // You can then process each item based on its type
    for (index, item) in mixed_array.items.iter().enumerate() {
        match item {
            JsonValue::Int(i) => println!("Item {} is an integer: {}", index, i),
            JsonValue::Str(s) => println!("Item {} is a string: {}", index, s),
            JsonValue::Bool(b) => println!("Item {} is a boolean: {}", index, b),
            _ => println!("Item {} is of another type", index),
        }
    }

    Ok(())
}
```

Other options here would have been to keep `items` as a [JsonArray], or even a [JsonValue]. Or, `items` could
be of type `Items` which has a manual implementation of [JsonDeserialize]. See the `mixed` example for inspiration.

## Deserializing types from other crates

You're going to need to use newtype wrappers: you can't implement `JsonSerializer`
(a type outside your crate) onto `time::OffsetDateTime` (also a type outside your crate),
as per the [orphan rules](https://github.com/Ixrec/rust-orphan-rules).

But you can implement it on `YourType<time::OffsetDateTime>` — and that works
especially well with date-time types, because, I like RFC3339, but you may want
to do something else.

The [merde_json_types](https://crates.io/crates/merde_json_types) crate aims to collect such wrapper
types: it's meant to be pulled unconditionally, and has a `merde_json` feature that conditionally
implements the relevant traits for the wrapper types, making it a cheap proposition if someone
wants to use your crate without using `merde_json`.

## Serializing

Serializing typically looks like:

```rust
# use merde_json::{Fantome, JsonSerialize, JsonDeserialize};
# use std::borrow::Cow;
#
# #[derive(Debug, PartialEq)]
# struct MyStruct<'s> {
#     _boo: Fantome<'s>,
#     name: Cow<'s, str>,
#     age: u8,
# }
#
# merde_json::derive! {
#     impl(JsonSerialize, JsonDeserialize) for MyStruct { name, age }
# }
#
# fn main() -> Result<(), merde_json::MerdeJsonError> {
let original = MyStruct {
    _boo: Default::default(),
    name: "John Doe".into(),
    age: 30,
};

let serialized = original.to_json_string();
println!("{}", serialized);

let ms: MyStruct = merde_json::from_str(&serialized)?;
assert_eq!(original, ms);
# Ok(())
# }
```

## Reducing allocations when serializing

If you want more control over the buffer, for example you'd like to re-use the same
`Vec<u8>` for multiple serializations, you can use [JsonSerializer::from_vec]:

```rust
# use merde_json::{Fantome, JsonSerialize, JsonDeserialize};
# use std::borrow::Cow;
#
# #[derive(Debug, PartialEq)]
# struct MyStruct<'s> {
#     _boo: Fantome<'s>,
#     name: Cow<'s, str>,
#     age: u8,
# }
#
# merde_json::derive! {
#     impl(JsonSerialize, JsonDeserialize) for MyStruct { name, age }
# }
#
# fn main() -> Result<(), merde_json::MerdeJsonError> {
let original = MyStruct {
    _boo: Default::default(),
    name: "John Doe".into(),
    age: 30,
};

let mut buffer = Vec::new();
for _ in 0..3 {
    buffer.clear();
    let mut serializer = merde_json::JsonSerializer::from_vec(buffer);
    original.json_serialize(&mut serializer);
    buffer = serializer.into_inner();

    let ms: MyStruct = merde_json::from_slice(&buffer)?;
    assert_eq!(original, ms);
}
# Ok(())
# }
```

Note that serialization is infallible, since it targest a memory buffer rather than
a Writer, and we assume allocations cannot fail (like most Rust code out there currently).

Keeping in mind that a `Vec` that grows doesn't give its memory back unless you ask for it
explicitly via [Vec::shrink_to_fit] or [Vec::shrink_to], for example.

## Caveats & limitations

Most of this crate is extremely naive, on purpose.

For example, deep data structures _will_ blow up the stack, since deserialization is recursive.

Deserialization round-trips through [jiter::JsonValue], which contains types like [std::sync::Arc],
small vecs, lazy hash maps, etc. — building them simply to destructure from them is a waste of CPU
cycles, and if it shows up in your profiles, it's time to move on to jiter's event-based parser,
[jiter::Jiter].

If you expect an `u32` but the JSON payload has a floating-point number, it'll get rounded.

If you expect a `u32` but the JSON payload is greater than `u32::MAX`, you'll get a
[MerdeJsonError::OutOfRange] error.

There's no control over allowing Infinity/NaN in JSON numbers: you can work around that
by calling [jiter::JsonValue::parse] yourself.

Serialization can't be pretty: it never produces unnecessary spaces, newlines, etc.
If your performance characteristics allow it, you may look into [formatjson](https://crates.io/crates/formatjson)

Serialization may produce JSON payloads that other parsers will reject or parse incorrectly,
specifically for numbers above 2^53 or below -2^53.

There is no built-in facility for serializing/deserializing strings from numbers.

If `merde_json` doesn't work for you, it's very likely that your use case is not supported, and
you should look at [serde](https://crates.io/crates/serde) instead.

## FAQ

### What's with the `Option` in the `JsonDeserialize` interface?

This allows `Option<T>` to ignore missing values. All other implementations should
return `MerdeJsonError::MissingValue` if the option is `None` — this is later turned
into `MerdeJsonError::MissingProperty` with the field name./

### What do I do about `#[serde(rename_all = "camelCase")]`?

Make your actual struct fields `camelCase`, and slap `#[allow(non_snake_case)]` on
top of your struct. Sorry!

### What do I do about `#[serde(borrow)]`?

That's the default and only mode — use `Cow<'a, str>` for all strings, do `.to_static()`
if you need to move the struct.
