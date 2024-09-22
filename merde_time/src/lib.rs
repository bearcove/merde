//! Provides [Rfc3339], a wrapper around [time::OffsetDateTime] that implements
//! [merde_json::JsonSerialize] and [merde_core::ValueDeserialize] when the right
//! cargo features are enabled.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

pub use time::OffsetDateTime;

/// A wrapper around date-time types that implements `JsonSerialize` and `JsonDeserialize`
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
    impl<'s> merde_core::ValueDeserialize<'s> for Rfc3339<time::OffsetDateTime> {
        fn from_value_ref<'val>(
            value: Option<&'val merde_core::Value<'s>>,
        ) -> Result<Self, merde_core::MerdeError<'s>> {
            let s = value
                .and_then(|v| v.as_str().ok())
                .ok_or(merde_core::MerdeError::MissingValue)?;
            Ok(Rfc3339(
                time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
                    .map_err(|_| merde_core::MerdeError::InvalidDateTimeValue)?,
            ))
        }

        fn from_value(
            value: Option<merde_core::Value<'s>>,
        ) -> Result<Self, merde_core::MerdeError<'s>> {
            match value {
                Some(v) => Self::from_value_ref(Some(&v)),
                None => Self::from_value_ref(None),
            }
        }
    }
}

#[cfg(feature = "json")]
mod merde_json_impls {
    use super::*;

    #[cfg(feature = "serialize")]
    impl merde_json::JsonSerialize for Rfc3339<time::OffsetDateTime> {
        fn json_serialize(&self, s: &mut merde_json::JsonSerializer) {
            // Note: we assume there's no need to escape the string
            let buf = s.as_mut_vec();
            buf.push(b'"');
            self.0
                .format_into(buf, &time::format_description::well_known::Rfc3339)
                .unwrap();
            buf.push(b'"');
        }
    }
}

#[cfg(all(test, feature = "full",))]
mod tests {
    use super::*;
    use merde_json::{from_str_via_value, JsonSerialize};
    use time::macros::datetime;

    #[test]
    fn test_rfc3339_offset_date_time_roundtrip() {
        let original = Rfc3339(datetime!(2023-05-15 14:30:00 UTC));
        let serialized = original.to_json_string();
        let deserialized: Rfc3339<time::OffsetDateTime> = from_str_via_value(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_rfc3339_offset_date_time_serialization() {
        let dt = Rfc3339(datetime!(2023-05-15 14:30:00 UTC));
        let serialized = dt.to_json_string();
        assert_eq!(serialized, r#""2023-05-15T14:30:00Z""#);
    }

    #[test]
    fn test_rfc3339_offset_date_time_deserialization() {
        let json = r#""2023-05-15T14:30:00Z""#;
        let deserialized: Rfc3339<time::OffsetDateTime> = from_str_via_value(json).unwrap();
        assert_eq!(deserialized, Rfc3339(datetime!(2023-05-15 14:30:00 UTC)));
    }
}
