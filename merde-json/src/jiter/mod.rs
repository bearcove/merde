//! This contains a stripped-down version of [jiter](https://crates.io/crates/jiter),
//! containing only their parsers/decoders and not their value types.

mod errors;
#[allow(clippy::module_inception)]
mod jiter;
mod number_decoder;
mod parse;
#[cfg(target_arch = "aarch64")]
mod simd_aarch64;
mod string_decoder;

pub(crate) use errors::{JiterError, JsonError, JsonErrorType, JsonResult};
pub(crate) use jiter::Jiter;
pub(crate) use number_decoder::NumberInt;
pub(crate) use parse::Peek;
