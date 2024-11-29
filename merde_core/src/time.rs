//! Provides [Rfc3339], a wrapper around [time::OffsetDateTime] that implements
//! [Serialize] and [Deserialize] when the right
//! cargo features are enabled.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::WithLifetime;

/// A wrapper around date-time types that implements `Serialize` and `Deserialize`
/// when the right cargo features are enabled.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Rfc3339<T>(pub T);

impl<T> WithLifetime<'_> for Rfc3339<T>
where
    T: 'static,
{
    type Lifetimed = Self;
}

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

#[cfg(feature = "time")]
pub use time::OffsetDateTime;

#[cfg(feature = "time")]
mod time_impls {
    use super::*;

    use time::OffsetDateTime;
    impl crate::IntoStatic for Rfc3339<OffsetDateTime> {
        type Output = Rfc3339<OffsetDateTime>;

        fn into_static(self) -> Self::Output {
            self
        }
    }

    impl<'s> crate::Deserialize<'s> for Rfc3339<time::OffsetDateTime> {
        async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
        where
            D: crate::Deserializer<'s> + ?Sized,
        {
            let s = crate::CowStr::deserialize(de).await?;
            Ok(Rfc3339(
                time::OffsetDateTime::parse(
                    s.as_ref(),
                    &time::format_description::well_known::Rfc3339,
                )
                .map_err(|_| crate::MerdeError::InvalidDateTimeValue)?,
            ))
        }
    }

    impl crate::Serialize for Rfc3339<time::OffsetDateTime> {
        async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where
            S: crate::Serializer + ?Sized,
        {
            let s = self
                .0
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap();
            serializer
                .write(crate::Event::Str(crate::CowStr::Borrowed(&s)))
                .await
        }
    }
}

#[cfg(all(test, feature = "full"))]
mod tests {
    use super::*;
    use crate::{Deserialize, Deserializer, Event, IntoStatic, MerdeError, Serializer};
    use std::collections::VecDeque;
    use time::macros::datetime;

    #[derive(Debug, Default)]
    struct Journal {
        events: VecDeque<Event<'static>>,
    }

    impl Serializer for Journal {
        type Error = std::convert::Infallible;

        async fn write(&mut self, event: Event<'_>) -> Result<(), Self::Error> {
            self.events.push_back(event.into_static());
            Ok(())
        }
    }

    impl<'s> Deserializer<'s> for Journal {
        type Error<'es> = MerdeError<'es>;

        fn next(&mut self) -> Result<Event<'s>, Self::Error<'s>> {
            Ok(self.events.pop_front().unwrap())
        }

        async fn t_starting_with<T: Deserialize<'s>>(
            &mut self,
            starter: Option<Event<'s>>,
        ) -> Result<T, Self::Error<'s>> {
            if let Some(event) = starter {
                self.events.push_front(event.into_static());
            }
            T::deserialize(self).await
        }
    }

    #[test]
    fn test_rfc3339_offset_date_time_roundtrip() {
        let original = Rfc3339(datetime!(2023-05-15 14:30:00 UTC));
        let mut journal: Journal = Default::default();

        journal.serialize_sync(&original).unwrap();
        let deserialized: Rfc3339<time::OffsetDateTime> = journal.deserialize_owned().unwrap();

        assert_eq!(original, deserialized);
    }

    // #[test]
    // fn test_rfc3339_offset_date_time_serialization() {
    //     let dt = Rfc3339(datetime!(2023-05-15 14:30:00 UTC));
    //     let serialized = dt.to_json_string().unwrap();
    //     assert_eq!(serialized, r#""2023-05-15T14:30:00Z""#);
    // }

    // #[test]
    // fn test_rfc3339_offset_date_time_deserialization() {
    //     let json = r#""2023-05-15T14:30:00Z""#;
    //     let deserialized: Rfc3339<time::OffsetDateTime> = from_str(json).unwrap();
    //     assert_eq!(deserialized, Rfc3339(datetime!(2023-05-15 14:30:00 UTC)));
    // }
}
