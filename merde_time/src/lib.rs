//! Provides [Rfc3339], a wrapper around [time::OffsetDateTime] that implements
//! [merde_core::Serialize] and [merde_core::Deserialize] when the right
//! cargo features are enabled.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

pub use time::OffsetDateTime;

/// A wrapper around date-time types that implements `Serialize` and `Deserialize`
/// when the right cargo features are enabled.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Rfc3339<T>(pub T);

impl<T> From<T> for Rfc3339<T> {
    fn from(t: T) -> Self {
        Rfc3339(t)
    }
}

impl<T> Deref for Rfc3339<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Rfc3339<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for Rfc3339<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> fmt::Display for Rfc3339<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "merde")]
mod merde_impls {
    use super::*;

    impl merde_core::IntoStatic for Rfc3339<OffsetDateTime> {
        type Output = Rfc3339<OffsetDateTime>;

        fn into_static(self) -> Self::Output {
            self
        }
    }

    #[cfg(feature = "deserialize")]
    impl<'s> merde_core::Deserialize<'s> for Rfc3339<time::OffsetDateTime> {
        async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
        where
            D: merde_core::Deserializer<'s> + ?Sized,
        {
            let s = merde_core::CowStr::deserialize(de).await?;
            Ok(Rfc3339(
                time::OffsetDateTime::parse(
                    s.as_ref(),
                    &time::format_description::well_known::Rfc3339,
                )
                .map_err(|_| merde_core::MerdeError::InvalidDateTimeValue)?,
            ))
        }
    }

    #[cfg(feature = "serialize")]
    impl merde_core::Serialize for Rfc3339<time::OffsetDateTime> {
        async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where
            S: merde_core::Serializer + ?Sized,
        {
            let s = self
                .0
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap();
            serializer
                .write(merde_core::Event::Str(merde_core::CowStr::Borrowed(&s)))
                .await
        }
    }
}

#[cfg(all(test, feature = "full"))]
mod tests {
    use super::*;
    use merde_json::{from_str, JsonSerialize};
    use time::macros::datetime;

    #[test]
    fn test_rfc3339_offset_date_time_roundtrip() {
        let original = Rfc3339(datetime!(2023-05-15 14:30:00 UTC));
        let serialized = original.to_json_string().unwrap();
        let deserialized: Rfc3339<time::OffsetDateTime> = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_rfc3339_offset_date_time_serialization() {
        let dt = Rfc3339(datetime!(2023-05-15 14:30:00 UTC));
        let serialized = dt.to_json_string().unwrap();
        assert_eq!(serialized, r#""2023-05-15T14:30:00Z""#);
    }

    #[test]
    fn test_rfc3339_offset_date_time_deserialization() {
        let json = r#""2023-05-15T14:30:00Z""#;
        let deserialized: Rfc3339<time::OffsetDateTime> = from_str(json).unwrap();
        assert_eq!(deserialized, Rfc3339(datetime!(2023-05-15 14:30:00 UTC)));
    }
}
