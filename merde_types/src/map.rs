use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{value::Value, CowStr, MerdeError, ValueDeserialize};

#[derive(Debug, PartialEq, Clone)]
#[repr(transparent)]
pub struct Map<'s>(pub HashMap<CowStr<'s>, Value<'s>>);

impl<'s> Map<'s> {
    pub fn new() -> Self {
        Map(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Map(HashMap::with_capacity(capacity))
    }

    pub fn with(mut self, key: impl Into<CowStr<'s>>, value: impl Into<Value<'s>>) -> Self {
        self.insert(key.into(), value.into());
        self
    }
}

impl<'s> Default for Map<'s> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> From<HashMap<CowStr<'s>, Value<'s>>> for Map<'s> {
    fn from(v: HashMap<CowStr<'s>, Value<'s>>) -> Self {
        Map(v)
    }
}

impl<'s> Deref for Map<'s> {
    type Target = HashMap<CowStr<'s>, Value<'s>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Map<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Map<'s> {
    /// Gets a value from the object, returning an error if the key is missing.
    ///
    /// Because this method knows the key name, it transforms [MerdeError::MissingValue] into [MerdeError::MissingProperty].
    ///
    /// It does not by itself throw an error if `self.get()` returns `None`, to allow
    /// for optional fields (via the [ValueDeserialize] implementation on the [Option] type).
    pub fn must_get<T>(&self, key: impl Into<CowStr<'static>>) -> Result<T, MerdeError>
    where
        T: ValueDeserialize<'s>,
    {
        let key = key.into();
        T::from_value_ref(self.get(&key)).map_err(|e| match e {
            MerdeError::MissingValue => MerdeError::MissingProperty(key),
            _ => e,
        })
    }

    /// Removes a value from the object, returning an error if the key is missing.
    ///
    /// Because this method knows the key name, it transforms [MerdeError::MissingValue] into [MerdeError::MissingProperty].
    ///
    /// It does not by itself throw an error if `self.remove()` returns `None`, to allow
    /// for optional fields (via the [ValueDeserialize] implementation on the [Option] type).
    pub fn must_remove<T>(&mut self, key: impl Into<CowStr<'static>>) -> Result<T, MerdeError>
    where
        T: ValueDeserialize<'s>,
    {
        let key = key.into();
        T::from_value(self.remove(&key)).map_err(|e| match e {
            MerdeError::MissingValue => MerdeError::MissingProperty(key),
            _ => e,
        })
    }
}
