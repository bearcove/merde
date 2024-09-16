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

    pub fn from_utf8_owned(s: Vec<u8>) -> Result<Self, std::str::Utf8Error> {
        #[cfg(feature = "compact_str")]
        {
            Ok(Self::Owned(CompactString::from_utf8(s)?))
        }
        #[cfg(not(feature = "compact_str"))]
        {
            Ok(String::from_utf8(s).map_err(|e| e.utf8_error())?.into())
        }
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

#[cfg(feature = "rusqlite")]
mod rusqlite_impls {
    use super::*;
    use rusqlite::{types::FromSql, types::FromSqlError, types::ToSql, Result as RusqliteResult};

    impl ToSql for CowStr<'_> {
        fn to_sql(&self) -> RusqliteResult<rusqlite::types::ToSqlOutput<'_>> {
            Ok(rusqlite::types::ToSqlOutput::Borrowed(self.as_ref().into()))
        }
    }

    impl FromSql for CowStr<'_> {
        fn column_result(value: rusqlite::types::ValueRef<'_>) -> Result<Self, FromSqlError> {
            match value {
                rusqlite::types::ValueRef::Text(s) => Ok(CowStr::from_utf8(s)
                    .map_err(|e| FromSqlError::Other(Box::new(e)))?
                    .into_static()),
                _ => Err(FromSqlError::InvalidType),
            }
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

    #[cfg(feature = "rusqlite")]
    #[test]
    fn test_rusqlite_integration() -> Result<(), Box<dyn std::error::Error>> {
        use rusqlite::Connection;

        // Create an in-memory database
        let conn = Connection::open_in_memory()?;

        // Create a table
        conn.execute(
            "CREATE TABLE test_table (id INTEGER PRIMARY KEY, value TEXT)",
            [],
        )?;

        // Insert a CowStr value
        let cow_str = CowStr::from("Hello, Rusqlite!");
        conn.execute("INSERT INTO test_table (value) VALUES (?1)", [&cow_str])?;

        // Retrieve the value
        let mut stmt = conn.prepare("SELECT value FROM test_table")?;
        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            let retrieved: CowStr = row.get(0)?;
            assert_eq!(retrieved, "Hello, Rusqlite!");
        } else {
            panic!("No rows returned");
        }

        Ok(())
    }
}
