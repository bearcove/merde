#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "time-types")]
pub mod time;

/// A type you can use instead of `PhantomData` for convenience.
///
/// This type is really just a convenience so you have less to type.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fantome<'s> {
    _boo: std::marker::PhantomData<&'s ()>,
}

impl std::fmt::Debug for Fantome<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Boo!")
    }
}
