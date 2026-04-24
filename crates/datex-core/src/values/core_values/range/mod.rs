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
