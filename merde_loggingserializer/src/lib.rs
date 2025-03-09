use merde_core::{Deserializer, Event, MerdeError};

pub struct LoggingDeserializer<'s, I>
where
    I: Deserializer<'s>,
{
    inner: I,
    starter: Option<Event<'s>>,
}

impl<'s, I> std::fmt::Debug for LoggingDeserializer<'s, I>
where
    I: Deserializer<'s>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoggingDeserializer")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'s, I> LoggingDeserializer<'s, I>
where
    I: Deserializer<'s>,
{
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            starter: None,
        }
    }
}

impl<'s, I> Deserializer<'s> for LoggingDeserializer<'s, I>
where
    I: Deserializer<'s>,
{
    async fn next(
        &mut self,
        type_hints: merde_core::TypeHints,
    ) -> Result<Event<'s>, MerdeError<'s>> {
        if let Some(ev) = self.starter.take() {
            eprintln!("> (from starter) {:?}", ev);
            return Ok(ev);
        }

        let ev = self.inner.next(type_hints).await?;
        eprintln!("> (from inner.next) {:?}", ev);
        Ok(ev)
    }

    fn put_back(&mut self, ev: Event<'s>) -> Result<(), MerdeError<'s>> {
        if self.starter.is_some() {
            return Err(MerdeError::PutBackCalledTwice);
        }
        self.starter = Some(ev);
        Ok(())
    }
}
