use std::{
    fmt,
    ops::{Deref, DerefMut},
};

/// A newtype wrapper around [time::OffsetDateTime] types, for which various traits
/// can be implemented:
///
///   - [crate::JsonSerialize] through the `time-serialize` feature
///   - [crate::JsonDeserialize] through the `time-deserialize` feature
///   - [crate::ToStatic] through the `time-types` feature (enabled by either of the above)
///
/// This wrapper tries to be as transparent as possible, implementing `Deref` and `DerefMut`,
/// forwarding `Debug and `Display`, etc.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Rfc3339<T>(T);

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

#[cfg(feature = "time-types")]
impl<T> crate::ToStatic for Rfc3339<T>
where
    T: crate::ToStatic,
{
    type Output = Rfc3339<<T as crate::ToStatic>::Output>;

    fn to_static(&self) -> Self::Output {
        Rfc3339(self.0.to_static())
    }
}

#[cfg(feature = "time-types")]
impl crate::ToStatic for time::OffsetDateTime {
    type Output = Self;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

#[cfg(feature = "time-serialize")]
impl crate::JsonSerialize for Rfc3339<time::OffsetDateTime> {
    fn json_serialize(&self, s: &mut crate::JsonSerializer) {
        // Note: we assume there's no need to escape the string
        let buf = s.as_mut_vec();
        buf.push(b'"');
        self.0
            .format_into(buf, &time::format_description::well_known::Rfc3339)
            .unwrap();
        buf.push(b'"');
    }
}

#[cfg(feature = "time-deserialize")]
impl<'src, 'val> crate::JsonDeserialize<'src, 'val> for Rfc3339<time::OffsetDateTime>
where
    'src: 'val,
{
    fn json_deserialize(
        value: Option<&'val crate::JsonValue<'src>>,
    ) -> Result<Self, crate::MerdeJsonError> {
        use crate::JsonValueExt;
        let s = value
            .and_then(|v| v.as_cow_str().ok())
            .ok_or(crate::MerdeJsonError::MissingValue)?;
        Ok(Rfc3339(
            time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
                .map_err(|_| crate::MerdeJsonError::InvalidDateTimeValue)?,
        ))
    }
}

#[cfg(all(test, feature = "time-serialize", feature = "time-deserialize"))]
mod tests {
    use crate::{from_str, JsonSerialize, ToRustValue};

    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_rfc3339_offset_date_time_roundtrip() {
        let original = Rfc3339(datetime!(2023-05-15 14:30:00 UTC));
        let serialized = original.to_json_string();
        let deserialized: Rfc3339<time::OffsetDateTime> =
            from_str(&serialized).unwrap().to_rust_value().unwrap();
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
        let deserialized: Rfc3339<time::OffsetDateTime> =
            from_str(json).unwrap().to_rust_value().unwrap();
        assert_eq!(deserialized, Rfc3339(datetime!(2023-05-15 14:30:00 UTC)));
    }
}
