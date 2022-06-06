//! MAKE THIS A STANDALONE CRATE

use std::{fmt::Display};
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct NonEmptyVector<T>(Vec<T>);

impl<T> NonEmptyVector<T> {
    pub fn new(elem: T) -> Self {
        NonEmptyVector(vec![elem])
    }
    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }
    pub fn last_mut(&mut self) -> &mut T {
        let elem = self.0.last_mut();
        unsafe { elem.unwrap_unchecked() }
    }

    pub fn last(&mut self) -> &T {
        let elem = self.0.last();
        return unsafe { elem.unwrap_unchecked() };
    }

    pub fn pop(&mut self) -> Result<T, AsSmallAsPossible> {
        if self.0.len() == 1 {
            return Err(AsSmallAsPossible);
        }
        let elem = self.0.pop();
        Ok(unsafe { elem.unwrap_unchecked() })
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> From<NonEmptyVector<T>> for Vec<T> {
    fn from(val: NonEmptyVector<T>) -> Self {
        val.0
    }
}

impl<T> IntoIterator for NonEmptyVector<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Error, Debug)]
pub struct AsSmallAsPossible;

impl Display for AsSmallAsPossible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "NonEmptyVector only has 1 element")
    }
}
