use crate::{
    prelude::*,
    preludes::derive::SharedReferencesCache,
    values::{
        borrowed_value_container::BorrowedValueContainer,
        value_container::ValueContainer,
        core_values::map::Map,
        core_values::list::List
    },
};

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
    Map(
        Box<
            dyn Iterator<
                    Item = (
                        BorrowedValueContainer<'a>,
                        BorrowedValueContainer<'a>,
                    ),
                > + 'a,
        >,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents the kind of parts that a value can be converted into.
pub enum PartsKind {
    /// Represents a list value (a struct without named fields).
    List,
    /// Represents a map value (a struct with named fields).
    Map,
    /// Represents no parts.
    None,
}

/// A trait for types that can be constructed from parts.
pub trait FromParts {
    /// Tries to construct the implementing type from parts.
    fn try_from_parts(_parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Err(())
    }
}

/// A trait for types that can be converted into parts.
pub trait IntoParts {
    /// Returns the kind of parts that the implementing type can be converted into.
    /// This can be used to check if the conversion is possible before attempting it.
    /// If this returns `PartsKind::None`, then the conversion is not possible.
    /// If this returns `PartsKind::List`, then the conversion is possible and `try_into_list_parts()` can be called.
    /// If this returns `PartsKind::Map`, then the conversion is possible and `try_into_map_parts()` can be called.
    fn parts_kind(&self) -> PartsKind {
        PartsKind::None
    }
    
    /// Converts the implementing type into its map parts.
    /// Returns an error if the conversion is not possible.
    /// You can check if the conversion is possible by calling `parts_kind()` 
    /// and checking if it returns `PartsKind::Map`.
    fn try_into_map_parts<'a>(
        self: Box<Self>,
        _cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()>
    where
        Self: 'a,
    {
        Err(())
    }
    
    /// Converts the implementing type into its list parts.
    /// Returns an error if the conversion is not possible.
    /// You can check if the conversion is possible by calling `parts_kind()` 
    /// and checking if it returns `PartsKind::List`.
    fn try_into_list_parts<'a>(
        self: Box<Self>,
        _cache: &'a mut SharedReferencesCache,
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        Err(())
    }
}