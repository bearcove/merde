use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use crate::{value::Value, CowStr, IntoStatic};

/// A map, dictionary, object, whatever — with string keys.
#[derive(PartialEq, Eq, Clone)]
#[repr(transparent)]
pub struct Map<'s>(pub HashMap<CowStr<'s>, Value<'s>>);

impl Hash for Map<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (k, v) in self.iter() {
            k.hash(state);
            v.hash(state);
        }
    }
}

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

impl Default for Map<'_> {
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
