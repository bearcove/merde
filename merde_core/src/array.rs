use std::ops::{Deref, DerefMut};

use crate::{value::Value, IntoStatic};

/// An array of [`Value`] items
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct Array<'s>(pub Vec<Value<'s>>);

impl<'s> Array<'s> {
    pub fn new() -> Self {
        Array(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Array(Vec::with_capacity(capacity))
    }

    pub fn into_inner(self) -> Vec<Value<'s>> {
        self.0
    }
}

impl std::fmt::Debug for Array<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl IntoStatic for Array<'_> {
    type Output = Array<'static>;

    #[inline(always)]
    fn into_static(self) -> <Self as IntoStatic>::Output {
        Array(self.0.into_iter().map(|v| v.into_static()).collect())
    }
}

impl<'s> IntoIterator for Array<'s> {
    type Item = Value<'s>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'s> Default for Array<'s> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> From<Vec<Value<'s>>> for Array<'s> {
    fn from(v: Vec<Value<'s>>) -> Self {
        Array(v)
    }
}

impl<'s> Deref for Array<'s> {
    type Target = Vec<Value<'s>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> DerefMut for Array<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Array<'s> {
    /// Pushes a value onto the back of the array.
    pub fn with(mut self, value: impl Into<Value<'s>>) -> Self {
        self.push(value.into());
        self
    }
}
