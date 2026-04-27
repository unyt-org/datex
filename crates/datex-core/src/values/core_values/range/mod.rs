//! This module contains the implementation of the [Range] struct, which represents a range of values in the type system.
//! A [Range] consists of a lower bound (inclusive) and an upper bound (exclusive) and can be used to represent ranges of numbers, ... (TBD)

use crate::values::value_container::ValueContainer;
use alloc::boxed::Box;
use core::fmt;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Range {
    // lower bound (inclusive)
    pub start: Box<ValueContainer>,
    // upper bound (exclusive)
    pub end: Box<ValueContainer>,
}

impl fmt::Debug for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        core::write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        core::write!(f, "{}..{}", self.start, self.end)
    }
}
