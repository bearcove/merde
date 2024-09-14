use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};

#[cfg(feature = "compact_str")]
use compact_str::CompactString;

use crate::IntoStatic;

/// A copy-on-write string type that uses [`CompactString`] for
/// the "owned" variant.
///
/// The standard [`Cow`] type cannot be used, since
/// `<str as ToOwned>::Owned` is `String`, and not `CompactString`.
#[derive(Clone)]
pub enum CowStr<'s> {
    Borrowed(&'s str),
    #[cfg(feature = "compact_str")]
    Owned(CompactString),
    #[cfg(not(feature = "compact_str"))]
    Owned(String),
}

impl<'s> CowStr<'s> {
    pub fn from_utf8(s: &'s [u8]) -> Result<Self, std::str::Utf8Error> {
        Ok(Self::Borrowed(std::str::from_utf8(s)?))
    }

    pub fn from_utf8_lossy(s: &'s [u8]) -> Self {
        #[cfg(feature = "compact_str")]
        {
            Self::Owned(CompactString::from_utf8_lossy(s))
        }
        #[cfg(not(feature = "compact_str"))]
        {
            String::from_utf8_lossy(s).into()
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes are valid UTF-8.
    pub unsafe fn from_utf8_unchecked(s: &'s [u8]) -> Self {
        #[cfg(feature = "compact_str")]
        {
            Self::Owned(CompactString::from_utf8_unchecked(s))
        }
        #[cfg(not(feature = "compact_str"))]
        {
            Self::Borrowed(std::str::from_utf8_unchecked(s))
        }
    }
}

impl AsRef<str> for CowStr<'_> {
    fn as_ref(&self) -> &str {
        match self {
            CowStr::Borrowed(s) => s,
            CowStr::Owned(s) => s.as_str(),
        }
    }
}

impl Deref for CowStr<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            CowStr::Borrowed(s) => s,
            CowStr::Owned(s) => s.as_str(),
        }
    }
}

impl<'a> From<Cow<'a, str>> for CowStr<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        match s {
            Cow::Borrowed(s) => CowStr::Borrowed(s),
            #[allow(clippy::useless_conversion)]
            Cow::Owned(s) => CowStr::Owned(s.into()),
        }
    }
}

impl<'s> From<&'s str> for CowStr<'s> {
    fn from(s: &'s str) -> Self {
        CowStr::Borrowed(s)
    }
}

impl From<String> for CowStr<'_> {
    fn from(s: String) -> Self {
        #[allow(clippy::useless_conversion)]
        CowStr::Owned(s.into())
    }
}

impl From<Box<str>> for CowStr<'_> {
    fn from(s: Box<str>) -> Self {
        CowStr::Owned(s.into())
    }
}

impl<'s> From<&'s String> for CowStr<'s> {
    fn from(s: &'s String) -> Self {
        CowStr::Borrowed(s.as_str())
    }
}

impl From<CowStr<'_>> for String {
    fn from(s: CowStr<'_>) -> Self {
        match s {
            CowStr::Borrowed(s) => s.into(),
            #[allow(clippy::useless_conversion)]
            CowStr::Owned(s) => s.into(),
        }
    }
}

impl From<CowStr<'_>> for Box<str> {
    fn from(s: CowStr<'_>) -> Self {
        match s {
            CowStr::Borrowed(s) => s.into(),
            CowStr::Owned(s) => s.into(),
        }
    }
}

impl<'a, 'b> PartialEq<CowStr<'a>> for CowStr<'b> {
    fn eq(&self, other: &CowStr<'a>) -> bool {
        self.deref() == other.deref()
    }
}

impl PartialEq<&str> for CowStr<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<CowStr<'_>> for &str {
    fn eq(&self, other: &CowStr<'_>) -> bool {
        *self == other.deref()
    }
}

impl PartialEq<String> for CowStr<'_> {
    fn eq(&self, other: &String) -> bool {
        self.deref() == other.as_str()
    }
}

impl PartialEq<CowStr<'_>> for String {
    fn eq(&self, other: &CowStr<'_>) -> bool {
        self.as_str() == other.deref()
    }
}

impl Eq for CowStr<'_> {}

impl Hash for CowStr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl fmt::Debug for CowStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl fmt::Display for CowStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl IntoStatic for CowStr<'_> {
    type Output = CowStr<'static>;

    fn into_static(self) -> Self::Output {
        match self {
            CowStr::Borrowed(s) => CowStr::Owned((*s).into()),
            CowStr::Owned(s) => CowStr::Owned(s),
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use super::*;

    use serde::{Deserialize, Serialize};

    impl Serialize for CowStr<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self)
        }
    }

    impl<'de> Deserialize<'de> for CowStr<'_> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[cfg(feature = "compact_str")]
            let s = compact_str::CompactString::deserialize(deserializer)?;

            #[cfg(not(feature = "compact_str"))]
            let s = String::deserialize(deserializer)?;

            Ok(CowStr::Owned(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partialeq_with_str() {
        let cow_str1 = CowStr::Borrowed("hello");
        let cow_str2 = CowStr::Borrowed("hello");
        let cow_str3 = CowStr::Borrowed("world");

        assert_eq!(cow_str1, "hello");
        assert_eq!("hello", cow_str1);
        assert_eq!(cow_str1, cow_str2);
        assert_ne!(cow_str1, "world");
        assert_ne!("world", cow_str1);
        assert_ne!(cow_str1, cow_str3);
    }
}
