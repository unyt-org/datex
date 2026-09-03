use crate::preludes::derive::SharedReferencesCache;
use crate::values::borrowed_value_container::BorrowedValueContainer;
use crate::values::value_container::ValueContainer;
use crate::prelude::*;

/// Represents the different parts of a disassembled value
/// that can be used to reconstruct the original value.
pub enum Parts<'a> {
    /// The parts of a list value (a struct without named fields).
    List(Box<dyn Iterator<Item = ValueContainer> + 'a>),
    /// The parts of a map value (a struct with named fields).
    Map(Box<dyn Iterator<Item = (ValueContainer, ValueContainer)> + 'a>),
}

/// Represents the different parts of a disassembled borrowed value.
pub enum BorrowedParts<'a> {
    List(Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>),
    Map(Box<dyn Iterator<Item = (BorrowedValueContainer<'a>, BorrowedValueContainer<'a>)> + 'a>),
}

/// A trait for types that can be constructed from parts.
pub trait FromParts {
    /// Tries to construct the implementing type from parts.
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized {
        Err(())
    }
}

/// A trait for types that can be converted into parts.
pub trait IntoParts {
    /// Converts the implementing type into its parts.
    fn into_parts<'a>(self, cache: &'a mut SharedReferencesCache) -> Option<Parts<'a>>
    where
        Self: Sized + 'a,
    {
        None
    }

    /// Converts the implementing type into its borrowed parts.
    fn as_parts<'a>(&'a self, cache: &'a mut SharedReferencesCache) -> Option<BorrowedParts<'a>> {
        None
    }
}