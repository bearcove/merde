[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/merde.svg)](https://crates.io/crates/merde)
[![docs.rs](https://docs.rs/merde/badge.svg)](https://docs.rs/merde)

# merde

![The merde logo: a glorious poop floating above a pair of hands](https://github.com/user-attachments/assets/763d60e0-5101-48af-bc72-f96f516a5d0f)

`merde` aims to provide a simpler, lighter alternative to [serde](https://crates.io/crates/serde),
that might run slower, but compiles faster.

This is the "hub" crate, which re-exports all types from [merde_core](https://crates.io/crates/merde_core),
including [`Value`], [`Array`], and [`Map`], and provides a declarative [`derive`] macro that helps implement
traits like [`Deserialize`], [`IntoStatic`] and [`Serialize`].

## Support matrix

| Format | Deserialize | Serialize | Production-ready? |
|--------|-------------|-----------|------------------|
| [JSON](https://crates.io/crates/merde_json) | 👍 | 👍 | lol no — please no |
| [YAML](https://crates.io/crates/merde_yaml) | 👍 | 🙅‍♀️ | still experimenting |
| [MsgPack](https://crates.io/crates/merde_msgpack) | 👍 | 🙅‍♀️ | here be dragons etc. |

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

let point = Point { x: 1, y: 2 };

let serialized = serde_json::to_string(&point).unwrap();
println!("serialized = {}", serialized);

let deserialized: Point = serde_json::from_str(&serialized).unwrap();
println!("deserialized = {:?}", deserialized);
```

By contrast, `merde` provides declarative macros — impls for traits
like `Deserialize`, `Serialize` can be generated with `merde::derive!`:

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

merde::derive! {
    impl (Deserialize, Serialize) for struct Point { x, y }
}

let point = Point { x: 1, y: 2 };

// note: `merde_json` is re-exported as `merde::json` if `merde`'s `json` feature is enabled
let serialized = merde::json::to_string(&point).unwrap();
println!("serialized = {}", serialized);

let deserialized: Point = merde::json::from_str(&serialized).unwrap();
println!("deserialized = {:?}", deserialized);
```

This approach is less flexible, but because there's no proc-macro involved, or
re-parsing of the struct definitions by the proc macro, it builds faster.

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

`merde` only handles "simple" cases: structs without a lifetime parameter
are the simplest, since they're always owned / `'static` (by definition):

```rust
#[derive(Debug)]
struct Name {
    first: String,
    middle: String,
    last: String,
}

merde::derive! {
    impl (Deserialize, Serialize) for struct Name { first, middle, last }
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
    //                                                     👇
    impl (Deserialize, Serialize) for struct Name<'s> { first, middle, last }
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

As of merde v5, "transparent" tuple structs are supported (aka "newtypes"):

```rust
use merde::CowStr;

#[derive(Debug)]
struct Wrapper<'s>(CowStr<'s>);

merde::derive! {
    impl (Deserialize, Serialize)
    for struct Wrapper<'s> transparent
}

let input = r#"["Hello, World!"]"#;
let wrapper: Vec<Wrapper> = merde::json::from_str(input).unwrap();
println!("Wrapped value: {:?}", wrapper);
```

Enums are also supported, only externally-tagged ones for now, and you need
to specify field names and variant names:

```rust
#[derive(Debug)]
enum TestEvent {
    MouseUp(MouseUp),
    MouseDown(MouseDown),
}

merde::derive! {
    impl (Serialize, Deserialize) for enum TestEvent
    externally_tagged {
        "mouseup" => MouseUp,
        "mousedown" => MouseDown,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MouseUp {
    x: i32,
    y: i32,
}

merde::derive! {
    impl (Serialize, Deserialize) for struct MouseUp {
        x,
        y
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MouseDown {
    x: i32,
    y: i32,
}

merde::derive! {
    impl (Serialize, Deserialize) for struct MouseDown {
        x,
        y
    }
}

let input = r#"{"mouseup": {"x": 100, "y": 200}}"#;
let event: TestEvent = merde::json::from_str(input).unwrap();
println!("TestEvent: {:?}", event);
```

"string-like" enums are also supported, like so:

```rust
#[derive(Debug)]
enum Emergency {
    Cuddle,
    Smoothie,
    Naptime,
    Playtime,
}

merde::derive! {
    impl (Serialize, Deserialize) for enum Emergency
    string_like {
        "cuddle" => Cuddle,
        "smoothie" => Smoothie,
        "naptime" => Naptime,
        "playtime" => Playtime,
    }
}

let input = r#"["cuddle", "smoothie", "playtime"]"#;
let emergencies: Vec<Emergency> = merde::json::from_str(input).unwrap();
println!("Emergencies: {:?}", emergencies);
```

### Interlude: why not `&'s str`?

You'll notice that `Deserialize` is not implemented for `&'s str`, ie.
this code does not compile:

```rust,compile_fail
#[derive(Debug)]
struct Name<'s> {
    first: &'s str,
    middle: &'s str,
    last: &'s str,
}

merde::derive! {
    impl (Deserialize, Serialize) for struct Nam<'s> { first, middle, last }
}
```

```text
error[E0277]: the trait bound `&str: Deserialize<'_>` is not satisfied
  --> merde/src/lib.rs:183:1
   |
12 | / merde::derive! {
13 | |     impl (Deserialize, Serialize) for struct Name<'s> { first, middle, last }
14 | | }
   | |_^ the trait `Deserialize<'_>` is not implemented for `&str`
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
use merde::{CowStr, Deserialize};

let input = r#"
    ["\"The Rock\""]
"#;

let v: Vec<CowStr<'_>> = merde::json::from_str(input).unwrap();
assert!(matches!(v.first().unwrap(), CowStr::Owned(_)));
```

Whereas something without escape sequences will end up being borrowed:

```rust
use merde::{CowStr, Deserialize};

let input = r#"
    ["Joever"]
"#;

let v: Vec<CowStr<'_>> = merde::json::from_str(input).unwrap();
assert!(matches!(v.first().unwrap(), CowStr::Borrowed(_)));
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
    impl (Deserialize, Serialize)
    for struct Message<'s> { kind, payload }
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
    let message: Message = merde::json::from_str(&s).unwrap();
    message
}

let _msg = recv_and_deserialize();
```

This fails to build with:

```text
error[E0515]: cannot return value referencing local variable `s`
    --> merde/src/lib.rs:366:9
    |
365 |         let message: Message = merde::json::from_str(&s).unwrap();
    |                                                      -- `s` is borrowed here
366 |         message
    |         ^^^^^^^ returns a value referencing data owned by the current function
```

That's where the `IntoStatic` trait comes in! Which you get for free when "deriving" `Deserialize`!

```rust
use merde::IntoStatic;
use merde::CowStr;

struct Message<'s> {
    kind: u8,
    payload: CowStr<'s>,
}

merde::derive! {
    impl (Deserialize, Serialize)
    for struct Message<'s> { kind, payload }
}

fn recv_and_deserialize() -> Message<'static> {
    let s: String = {
        // pretend this reads from the network instead, or something
        r#"{
            "kind": 42,
            "payload": "hello"
        }"#.to_owned()
    };
    let message: Message = merde::json::from_str(&s).unwrap();
    message.into_static()
}

let _msg = recv_and_deserialize();
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
use merde::CowStr;
use merde::time::{Rfc3339, OffsetDateTime};

#[derive(Debug)]
struct Person<'s> {
    name: CowStr<'s>,
    birth: Rfc3339<OffsetDateTime>,
}

merde::derive! {
    impl (Deserialize, Serialize) for struct Person<'s> { name, birth }
}

let input = r#"
    {
        "name": "Jane Smith",
        "birth": "1990-01-01T00:00:00Z"
    }
"#;

let person: Person = merde::json::from_str(input).unwrap();
println!("person = {:?}", person);
```

You can of course make your own newtype wrappers to control how a field gets deserialized.

## Conditional compilation

(As of merde 3.1), you never need to add `cfg` gates to conditionally invoke the `merde::derive!`
macro, because, with default features disabled, `merde` has zero dependencies.

There's two main ways to be conservative with the amount of generated code / the amount of
dependencies pulled with merde.

### Approach 1: "core" by default, "deserialize" on demand

Your manifest could look like this:

```toml
# in `Cargo.toml`

[dependencies]
merde = { version = "4.0.0", default-features = false, features = ["core"] }
```

And then you'd be able to use merde_provided types, like `CowStr`:

```rust
use merde::CowStr;

#[derive(Debug)]
struct Person<'s> {
    name: CowStr<'s>,
    age: u8, // sorry 256-year-olds
}

merde::derive! {
    impl (Deserialize, Serialize) for struct Person<'s> { name, age }
}
```

And the `impl` blocks for `Deserialize`, and `Serialize` wouldn't actually
be generated unless crates downstream of yours enable `merde/deserialize` or `merde/json`.

### Approach 2: zero-deps

If your manifest looks more like this:

```toml
# in `Cargo.toml`

[dependencies]
merde = { version = "4.0.0", default-features = false }

[features]
default = []
merde = ["merde/core"]
```

...with no `merde` features enabled by default at all, then you have to stay
away from merde types, or use substitutes, for example, you could switch
`CowStr<'s>` with `std::borrow::Cow<'s, str>` and get largely the same API:

```rust
#[cfg(feature = "merde")]
use merde::CowStr;

#[cfg(not(feature = "merde"))]
pub type CowStr<'s> = std::borrow::Cow<'s, str>;

#[derive(Debug)]
pub struct Person<'s> {
    name: CowStr<'s>,
    age: u8, // sorry 256-year-olds
}

merde::derive! {
    impl (Deserialize, Serialize) for struct Person<'s> { name, age }
}
```

(But not the same ABI! Careful if you use this in conjunction with something
like [rubicon](https://github.com/bearcove/rubicon)).

With that configuration, users of your crate will only have to pay for downloading
`merde` and evaluating a few `derive!` macros which will produce empty token trees —
no extra dependencies, barely any extra build time.

See `zerodeps-example` in the [merde repository](https://github.com/bearcove/merde)
for a demonstration:

```shell
❯ cargo tree --prefix none
zerodeps-example v0.1.0 (/Users/amos/bearcove/merde/zerodeps-example)
merde v3.0.0 (/Users/amos/bearcove/merde/merde)
```

```shell
❯ cargo tree --prefix none --features 'merde'
zerodeps-example v0.1.0 (/Users/amos/bearcove/merde/zerodeps-example)
merde v3.0.0 (/Users/amos/bearcove/merde/merde)
merde_core v3.0.0 (/Users/amos/bearcove/merde/merde_core)
compact_str v0.8.0
castaway v0.2.3
rustversion v1.0.17 (proc-macro)
cfg-if v1.0.0
itoa v1.0.11
rustversion v1.0.17 (proc-macro)
ryu v1.0.18
static_assertions v1.1.0
```
