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
    use std::future::Future;

    use super::*;

    use time::OffsetDateTime;
    impl crate::IntoStatic for Rfc3339<OffsetDateTime> {
        type Output = Rfc3339<OffsetDateTime>;

        fn into_static(self) -> Self::Output {
            self
        }
    }

    impl<'s> crate::Deserialize<'s> for Rfc3339<time::OffsetDateTime> {
        async fn deserialize(
            de: &mut dyn crate::DynDeserializer<'s>,
        ) -> Result<Self, crate::MerdeError<'s>> {
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
        #[allow(clippy::manual_async_fn)]
        fn serialize<'fut>(
            &'fut self,
            serializer: &'fut mut dyn crate::DynSerializer,
        ) -> impl Future<Output = Result<(), crate::MerdeError<'static>>> + 'fut {
            async move {
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
}

#[cfg(all(test, feature = "full"))]
mod tests {
    use super::*;
    use crate::{Deserializer, DynSerializerExt, Event, IntoStatic, MerdeError, Serializer};
    use std::{collections::VecDeque, future::Future};
    use time::macros::datetime;

    #[derive(Debug, Default)]
    struct Journal {
        events: VecDeque<Event<'static>>,
    }

    impl Serializer for Journal {
        async fn write<'fut>(
            &'fut mut self,
            event: Event<'fut>,
        ) -> Result<(), MerdeError<'static>> {
            self.events.push_back(event.into_static());
            Ok(())
        }
    }

    impl<'s> Deserializer<'s> for Journal {
        // FIXME: that's a workaround for <https://github.com/rust-lang/rust/issues/133676>
        #[allow(clippy::manual_async_fn)]
        fn next(&mut self) -> impl Future<Output = Result<Event<'s>, MerdeError<'s>>> + '_ {
            async { self.events.pop_front().ok_or_else(MerdeError::eof) }
        }

        fn put_back(&mut self, ev: Event<'s>) -> Result<(), MerdeError<'s>> {
            self.events.push_front(ev.into_static());
            Ok(())
        }
    }

    #[test]
    fn test_rfc3339_offset_date_time_roundtrip() {
        let original = Rfc3339(datetime!(2023-05-15 14:30:00 UTC));
        let mut journal: Journal = Default::default();

        use crate::DynDeserializerExt;

        journal.serialize(&original).unwrap();
        let deserialized = journal
            .deserialize_owned::<Rfc3339<time::OffsetDateTime>>()
            .unwrap();

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
