#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "time-types")]
pub mod time;

/// A type you can use instead of `PhantomData` for convenience.
///
/// This type is really just a convenience so you have less to type.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fantome<'src, 'val> {
    _boo: std::marker::PhantomData<(&'src (), &'val ())>,
}

impl std::fmt::Debug for Fantome<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Boo!")
    }
}
