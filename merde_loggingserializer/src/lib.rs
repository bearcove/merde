use merde_core::{Deserialize, Deserializer, Event};

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
    type Error<'es> = I::Error<'es>;

    fn next(&mut self) -> Result<Event<'s>, Self::Error<'s>> {
        if let Some(ev) = self.starter.take() {
            eprintln!("> (from starter) {:?}", ev);
            return Ok(ev);
        }

        let ev = self.inner.next()?;
        eprintln!("> (from inner.next) {:?}", ev);
        Ok(ev)
    }

    async fn t_starting_with<T: Deserialize<'s>>(
        &mut self,
        starter: Option<Event<'s>>,
    ) -> Result<T, Self::Error<'s>> {
        if let Some(starter) = starter {
            if self.starter.is_some() {
                unreachable!("setting starter when it's already set? shouldn't happen")
            }
            self.starter = Some(starter);
        }

        T::deserialize(self).await
    }
}
