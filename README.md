[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/merde_json.svg)](https://crates.io/crates/merde_json)
[![docs.rs](https://docs.rs/merde_json/badge.svg)](https://docs.rs/merde_json)

![The merde_json logo: a glorious poop floating above a pair of hands](assets/merde-json-banner@2x.png)

_Logo by [Misiasart](https://www.deviantart.com/misiasart)_

# merde_json

Do you want to deal with JSON data? Are you not _that_ worried about the
performance overhead? (ie. you're writing a backend in Rust, but if it was
written in Node.js nobody would bat an eye).

Do you value short build times at the expense of some comfort?

Then head over to the crate documentations:

  * [merde_json](./merde_json/README.md)
  * [merde_json_types](./merde_json_types/README.md)

## FAQ

### What's with the name?

Tis true "merde" means "poop" in French, but it's also one letter away from "serde".

Just pretending we're merializing and demerializing things.

### Can we do YAML/TOML/etc.?

"We" won't, but you certainly can! In another crate!

`merde_json` has JSON in the name specifically because it's not interested in solving
other formats. Use serde for that, or whatever crate is backing the `serde_FORMAT` you're interested in.

Currently, `merde_json` depends on `jiter::JsonValue` and moving away from that would be
a whole headache.

### Small?? That's a lot of deps still

I mean, fair, `jiter` pulls a bunch of deps:

```
❯ cargo tree -e normal
merde_json v1.0.0 (/Users/amos/bearcove/merde_json)
└── jiter v0.5.0
    ├── ahash v0.8.11
    │   ├── cfg-if v1.0.0
    │   ├── getrandom v0.2.15
    │   │   ├── cfg-if v1.0.0
    │   │   └── libc v0.2.155
    │   ├── once_cell v1.19.0
    │   └── zerocopy v0.7.35
    ├── bitvec v1.0.1
    │   ├── funty v2.0.0
    │   ├── radium v0.7.0
    │   ├── tap v1.0.1
    │   └── wyz v0.5.1
    │       └── tap v1.0.1
    ├── lexical-parse-float v0.8.5
    │   ├── lexical-parse-integer v0.8.6
    │   │   ├── lexical-util v0.8.5
    │   │   │   └── static_assertions v1.1.0
    │   │   └── static_assertions v1.1.0
    │   ├── lexical-util v0.8.5 (*)
    │   └── static_assertions v1.1.0
    ├── num-bigint v0.4.6
    │   ├── num-integer v0.1.46
    │   │   └── num-traits v0.2.19
    │   └── num-traits v0.2.19
    ├── num-traits v0.2.19
    └── smallvec v1.13.2
```

But also, the whole thing builds in 1.88s on my machine. And you only pay for those once,
unlike proc macros, which currently (as of Rust 1.80 at least) have a great compile-time
penalty, cache poorly, etc.

Also, [bitvec](https://crates.io/crates/bitvec) is awesome, [ahash](https://crates.io/crates/ahash)
is awesome, BigInt support could probably be made opt-in?

Let's see if the `jiter` authors get in touch, it's a pretty recent crate.

### Can you support feature X that serde has?

Probably not. The answer is probably "implement the Serialize/Deserialize traits
manually" (much easier than the serde equivalent, thankfully), or "just use the
serde ecosystem".

### Wouldn't it be a lot more flexible to use a code generator rather than declarative macros?

Yes it would. I'd love that. But this is a nice first step.
