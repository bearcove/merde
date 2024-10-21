use std::future::Future;

use crate::Event;

pub trait Serializer {
    type Error;

    // (note: this is an async fn but because there's a lifetime, it won't let us!)
    fn write(&mut self, ev: Event<'_>) -> impl Future<Output = Result<(), Self::Error>>;
}

impl<S> Serializer for &mut S
where
    S: Serializer,
{
    type Error = S::Error;

    /// Write the next event
    async fn write(&mut self, ev: Event<'_>) -> Result<(), Self::Error> {
        <S as Serializer>::write(self, ev).await
    }
}

pub trait Serialize {
    #[allow(async_fn_in_trait)]
    async fn serialize<S: Serializer>(&self, serializer: S) -> Result<(), S::Error>;
}

impl Serialize for i64 {
    async fn serialize<S: Serializer>(&self, mut serializer: S) -> Result<(), S::Error> {
        serializer.write(Event::I64(*self)).await
    }
}
