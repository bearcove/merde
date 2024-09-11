[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/merde.svg)](https://crates.io/crates/merde)
[![docs.rs](https://docs.rs/merde/badge.svg)](https://docs.rs/merde)

# merde

[The merde logo: a glorious poop floating above a pair of hands](https://github.com/user-attachments/assets/763d60e0-5101-48af-bc72-f96f516a5d0f)

`merde` aims to provide a simpler, lighter alternative to [serde](https://crates.io/crates/serde),
that might run slower, but compiles faster.

This is the "hub" crate, which re-exports all types from [merde_core](https://crates.io/crates/merde_core),
including [`Value`], [`Array`], and [`Map`], and provides a declarative [`derive`] macro that helps implement
traits like [`ValueDeserialize`], [`IntoStatic`] and [`JsonSerialize`].

## From `serde` to `merde`

### Deriving impls, serializing, deserializing

`serde` has its own `Serialize` and `Deserialize` traits, which you can derive
with, well, derive macros:

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    let serialized = serde_json::to_string(&point).unwrap();
    println!("serialized = {}", serialized);

    let deserialized: Point = serde_json::from_str(&serialized).unwrap();
    println!("deserialized = {:?}", deserialized);
}
```

By contrast, `merde` provides declarative macros — impls for traits
like `ValueDeserialize`, `JsonSerialize` can be generated with `merde::derive!`:

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

merde::derive! {
    impl (ValueDeserialize, JsonSerialize) for Point { x, y }
}

fn main() {
    let point = Point { x: 1, y: 2 };

    // note: `merde_json` is re-exported as `merde::json` if `merde`'s `json` feature is enabled
    let serialized = merde::json::to_string(&point);
    println!("serialized = {}", serialized);

    let deserialized: Point = merde::json::from_str_via_value(&serialized).unwrap();
    println!("deserialized = {:?}", deserialized);
}
```

This approach is less flexible, but because there's no proc-macro involved, or
re-parsing of the struct definitions by the proc macro, it builds faster.

[`json::from_str_via_value`] round-trips through [`Value`] but that's not inherent
to the merde approach, we just need to figure out the right approach.

### Copy-on-write types

Picture this: a large JSON documents, with large strings, that don't use escape sequences.

Instead of allocating a separate `String` on the heap for each of these, `serde` lets you
borrow from the input string, either automatically when you use a type like `&str`:

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Name<'s> {
    first: &'s str,
    middle: &'s str,
    last: &'s str,
}

// etc.
```

Or explicitly when you use a copy-on-write type like `Cow<'s, str>`:

```rust,ignore
use serde::{Serialize, Deserialize};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug)]
struct Name<'s> {
    #[serde(borrow)]
    first: Cow<'s, str>,
    #[serde(borrow)]
    middle: Cow<'s, str>,
    #[serde(borrow)]
    last: Cow<'s, str>,
}
```

`serde` is really flexible here, letting you have types with multiple lifetimes, not
all of which are related to the input string.

`merde` only handles the simplest of cases: structs without a lifetime parameter
are the simple case, since they're always owned / `'static` (by definition):

```rust
#[derive(Debug)]
struct Name {
    first: String,
    middle: String,
    last: String,
}

merde::derive! {
    impl (ValueDeserialize, JsonSerialize) for Name { first, middle, last }
}
```

But, as a treat, structs passed to `merde::derive!` can have exactly one lifetime
parameter, so that you may use copy-on-write types, like merde's own `CowStr`:

```rust
use merde::CowStr;

#[derive(Debug)]
struct Name<'s> {
    first: CowStr<'s>,
    middle: CowStr<'s>,
    last: CowStr<'s>,
}

merde::derive! {
    //                                              👇
    impl (ValueDeserialize, JsonSerialize) for Name<'s> { first, middle, last }
    //                                              👆
}
```

Note that in the `merde::derive!` invocation, we _have_ to give it the lifetime parameter's
name — this ends up generating different code, that can borrow from the input.

Although you may use [`Cow<'s, str>`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) merde
recommends [`CowStr<'s>`](https://docs.rs/merde/latest/merde/struct.CowStr.html) type,
which dereferences to `&str` like you'd expect, but instead of using
[`String`](https://doc.rust-lang.org/std/string/struct.String.html) as its owned type, it uses
[`compact_str::CompactString`](https://docs.rs/compact_str/0.8.0/compact_str/struct.CompactString.html),
which stores strings of up to 24 bytes inline!

### Interlude: why not `&'s str`?

You'll notice that `ValueDeserialize` is not implemented for `&'s str`, ie.
this code does not compile:

```rust,compile_fail
#[derive(Debug)]
struct Name<'s> {
    first: &'s str,
    middle: &'s str,
    last: &'s str,
}

merde::derive! {
    impl (ValueDeserialize, JsonSerialize) for Name<'s> { first, middle, last }
}
```

```text
error[E0277]: the trait bound `&str: ValueDeserialize<'_>` is not satisfied
  --> merde/src/lib.rs:183:1
   |
12 | / merde::derive! {
13 | |     impl (ValueDeserialize, JsonSerialize) for Name<'s> { first, middle, last }
14 | | }
   | |_^ the trait `ValueDeserialize<'_>` is not implemented for `&str`
```

That's because it's not always possible to borrow from the input.

This JSON input would be fine:

```json
{
  "first": "Jane",
  "middle": "Panic",
  "last": "Smith"
}
```

But this JSON input would not:

```json
{
  "first": "Jane",
  "middle": "\"The Rock\"",
  "last": "Smith",
}
```

We could borrow `"The Rock`, but then we'd have a problem: the next _actual_ character is a double-quote,
but the next character from the input is the backslash (`\`) used to escape the double-quote.

Such a string will end up being owned in a `CowStr`:

```rust
use merde::{CowStr, ValueDeserialize};

fn main() {
    let input = r#"
        ["\"The Rock\""]
    "#;

    let v: Vec<CowStr<'_>> = merde::json::from_str_via_value(input).unwrap();
    assert!(matches!(v.first().unwrap(), CowStr::Owned(_)));
}
```

Whereas something without escape sequences will end up being borrowed:

```rust
use merde::{CowStr, ValueDeserialize};

fn main() {
    let input = r#"
        ["Joever"]
    "#;

    let v: Vec<CowStr<'_>> = merde::json::from_str_via_value(input).unwrap();
    assert!(matches!(v.first().unwrap(), CowStr::Borrowed(_)));
}
```

All this is pretty JSON-specific, but you get the idea.

### Returning something you've serialized

Borrowing from the input (to avoid allocations and copies, in case you missed
the memo) is all fun and games until you need to move something around, for
example, returning it from a function.

```rust,compile_fail
use merde::CowStr;

struct Message<'s> {
    kind: u8,
    payload: CowStr<'s>,
}

merde::derive! {
    impl (ValueDeserialize, JsonSerialize)
    for Message<'s> { kind, payload }
}

// well this is already fishy, where does the `'s` come from?
fn recv_and_deserialize<'s>() -> Message<'s> {
    let s: String = {
        // pretend this reads from the network instead, or something
        r#"{
            "kind": 42,
            "payload": "hello"
        }"#.to_owned()
    };
    let message: Message = merde::json::from_str_via_value(&s).unwrap();
    message
}

fn main() {
    let _msg = recv_and_deserialize();
}
```

This fails to build with:

```text
error[E0515]: cannot return value referencing local variable `s`
    --> merde/src/lib.rs:366:9
    |
365 |         let message: Message = merde::json::from_str_via_value(&s).unwrap();
    |                                                                -- `s` is borrowed here
366 |         message
    |         ^^^^^^^ returns a value referencing data owned by the current function
```

That's where the `IntoStatic` trait comes from — which you can also derive
with `merde::derive!`:

```rust
use merde::IntoStatic;
use merde::CowStr;

struct Message<'s> {
    kind: u8,
    payload: CowStr<'s>,
}

merde::derive! {
    impl (ValueDeserialize, JsonSerialize, IntoStatic)
    for Message<'s> { kind, payload }
}

fn recv_and_deserialize() -> Message<'static> {
    let s: String = {
        // pretend this reads from the network instead, or something
        r#"{
            "kind": 42,
            "payload": "hello"
        }"#.to_owned()
    };
    let message: Message = merde::json::from_str_via_value(&s).unwrap();
    message.into_static()
}

fn main() {
    let _msg = recv_and_deserialize();
}
```

Et voilà! ✨

There might be something smarter to do based on the [yoke](https://docs.rs/yoke) crate for example,
but for now, allocations it is.

### Third-party types

Some crates don't have a `merde` features. In fact, at the time of this writing,
no crates at all do. `merde` is still moving fast (despite major features), so
in fact, I would encourage crate authors _not_ to ship a `merde` feature yet, it
would just create frustrating churn.

`serde` lets you work around that by specifying a function that should be used to
deserialize some field:

```rust,ignore
use serde::{Serialize, Deserialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    #[serde(with = "time::serde::rfc3339")]
    birth: OffsetDateTime,
}
```

Which solves two problems at once:

  1. the crate may not know about serde at all (not true in this case, time does have a serde feature)
  2. there might be several ways to serialize/deserialize something (RFC-3339, ISO-8601, and many others etc.)

`merde` solves both of these with wrapper types:

```rust
use time::OffsetDateTime;
use merde::CowStr;
use merde::time::Rfc3339;

#[derive(Debug)]
struct Person<'s> {
    name: CowStr<'s>,
    birth: Rfc3339<OffsetDateTime>,
}

merde::derive! {
    impl (ValueDeserialize, JsonSerialize) for Person<'s> { name, birth }
}

fn main() {
    let input = r#"
        {
            "name": "Jane Smith",
            "birth": "1990-01-01T00:00:00Z"
        }
    "#;

    let person: Person = merde::json::from_str_via_value(input).unwrap();
    println!("person = {:?}", person);
}
```

You can of course make your own newtype wrappers to control how a field gets deserialized.
