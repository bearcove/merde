//! This contains a stripped-down version of [jiter](https://crates.io/crates/jiter),
//! containing only their parsers/decoders and not their value types.

pub(crate) mod errors;
#[allow(clippy::module_inception)]
pub(crate) mod jiter;
pub(crate) mod number_decoder;
pub(crate) mod parse;
#[cfg(target_arch = "aarch64")]
pub(crate) mod simd_aarch64;
pub(crate) mod string_decoder;
