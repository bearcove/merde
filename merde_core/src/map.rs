use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{value::Value, CowStr, IntoStatic, MerdeError, ValueDeserialize};

/// A map, dictionary, object, whatever — with string keys.
#[derive(PartialEq, Clone)]
#[repr(transparent)]
pub struct Map<'s>(pub HashMap<CowStr<'s>, Value<'s>>);

impl std::fmt::Debug for Map<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

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

    pub fn into_inner(self) -> HashMap<CowStr<'s>, Value<'s>> {
        self.0
    }
}

impl IntoStatic for Map<'_> {
    type Output = Map<'static>;

    #[inline(always)]
    fn into_static(self) -> <Self as IntoStatic>::Output {
        Map(self
            .into_iter()
            .map(|(k, v)| (k.into_static(), v.into_static()))
            .collect())
    }
}

impl<'s> IntoIterator for Map<'s> {
    type Item = (CowStr<'s>, Value<'s>);
    type IntoIter = std::collections::hash_map::IntoIter<CowStr<'s>, Value<'s>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
    pub fn must_get<T>(&self, key: impl Into<CowStr<'static>>) -> Result<T, MerdeError<'s>>
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
    pub fn must_remove<T>(&mut self, key: impl Into<CowStr<'static>>) -> Result<T, MerdeError<'s>>
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
