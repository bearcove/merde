use std::ops::{Deref, DerefMut};

use crate::{value::Value, MerdeError, ValueDeserialize};

/// An array of [`Value`] items
#[derive(Debug, PartialEq, Clone)]
#[repr(transparent)]
pub struct Array<'s>(pub Vec<Value<'s>>);

impl<'s> Array<'s> {
    pub fn new() -> Self {
        Array(Vec::new())
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
    /// Gets a value from the array, returning an error if the index is out of bounds.
    ///
    /// Because this method knows the index, it transforms [MerdeError::MissingValue] into [MerdeError::IndexOutOfBounds].
    ///
    /// It does not by itself throw an error if `self.get()` returns `None`, to allow
    /// for optional fields (via the [ValueDeserialize] implementation on the [Option] type).
    pub fn must_get<T>(&self, index: usize) -> Result<T, MerdeError>
    where
        T: ValueDeserialize<'s>,
    {
        T::from_value_ref(self.get(index)).map_err(|e| match e {
            MerdeError::MissingValue => MerdeError::IndexOutOfBounds {
                index,
                len: self.len(),
            },
            _ => e,
        })
    }

    /// Pops a value from the back of the array and deserializes it
    pub fn must_pop<T>(&mut self) -> Result<T, MerdeError>
    where
        T: ValueDeserialize<'s>,
    {
        T::from_value(self.pop()).map_err(|e| match e {
            MerdeError::MissingValue => MerdeError::IndexOutOfBounds {
                index: self.len(),
                len: self.len(),
            },
            _ => e,
        })
    }

    /// Pushes a value onto the back of the array.
    pub fn with(mut self, value: impl Into<Value<'s>>) -> Self {
        self.push(value.into());
        self
    }
}
