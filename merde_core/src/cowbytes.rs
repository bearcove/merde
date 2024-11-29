use compact_bytes::CompactBytes;
use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};

use crate::IntoStatic;

/// A copy-on-write bytes type that uses [`CompactBytes`] for
/// the "owned" variant.
#[derive(Clone)]
pub enum CowBytes<'a> {
    Borrowed(&'a [u8]),
    Owned(CompactBytes),
}

impl<'a> CowBytes<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        CowBytes::Borrowed(bytes)
    }

    pub fn into_owned(self) -> Vec<u8> {
        match self {
            CowBytes::Borrowed(b) => b.to_vec(),
            CowBytes::Owned(b) => b.to_vec(),
        }
    }
}

impl AsRef<[u8]> for CowBytes<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            CowBytes::Borrowed(b) => b,
            CowBytes::Owned(b) => b.as_ref(),
        }
    }
}

impl Deref for CowBytes<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> From<&'a [u8]> for CowBytes<'a> {
    fn from(b: &'a [u8]) -> Self {
        CowBytes::Borrowed(b)
    }
}

impl From<Vec<u8>> for CowBytes<'_> {
    fn from(v: Vec<u8>) -> Self {
        CowBytes::Owned(CompactBytes::from(v))
    }
}

impl<'a> From<Cow<'a, [u8]>> for CowBytes<'a> {
    fn from(cow: Cow<'a, [u8]>) -> Self {
        match cow {
            Cow::Borrowed(b) => CowBytes::Borrowed(b),
            Cow::Owned(v) => v.into(),
        }
    }
}

impl<'a> PartialEq<CowBytes<'a>> for CowBytes<'_> {
    fn eq(&self, other: &CowBytes<'a>) -> bool {
        self.deref() == other.deref()
    }
}

impl PartialEq<[u8]> for CowBytes<'_> {
    fn eq(&self, other: &[u8]) -> bool {
        self.deref() == other
    }
}

impl PartialEq<CowBytes<'_>> for [u8] {
    fn eq(&self, other: &CowBytes<'_>) -> bool {
        self == other.deref()
    }
}

impl Eq for CowBytes<'_> {}

impl Hash for CowBytes<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl fmt::Debug for CowBytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

impl IntoStatic for CowBytes<'_> {
    type Output = CowBytes<'static>;

    fn into_static(self) -> Self::Output {
        match self {
            CowBytes::Borrowed(b) => CowBytes::Owned(CompactBytes::new(b)),
            CowBytes::Owned(b) => CowBytes::Owned(b),
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use super::*;
    use serde::{Deserialize, Serialize};

    impl Serialize for CowBytes<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_bytes(self)
        }
    }

    impl<'de> Deserialize<'de> for CowBytes<'_> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let bytes = Vec::<u8>::deserialize(deserializer)?;
            Ok(CowBytes::from(bytes))
        }
    }
}
