use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};

use compact_str::CompactString;

/// A copy-on-write string type that uses [CompactString] for
/// the "owned" variant.
///
/// The standard [std::borrow::Cow] type cannot be used, since
/// `<str as ToOwned>::Owned` is `String`, and not `CompactString`.
#[derive(Clone)]
pub enum CowStr<'s> {
    Borrowed(&'s str),
    Owned(CompactString),
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
            Cow::Owned(s) => CowStr::Owned(CompactString::from(s)),
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
        CowStr::Owned(CompactString::from(s))
    }
}

impl From<Box<str>> for CowStr<'_> {
    fn from(s: Box<str>) -> Self {
        CowStr::Owned(CompactString::from(s))
    }
}

impl<'s> From<&'s String> for CowStr<'s> {
    fn from(s: &'s String) -> Self {
        CowStr::Borrowed(s.as_str())
    }
}

impl PartialEq for CowStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
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
