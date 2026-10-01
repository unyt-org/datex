use crate::{
    prelude::*,
    preludes::derive::SharedReferencesCache,
    values::{
        borrowed_value_container::BorrowedValueContainer,
        core_values::{list::List, map::Map},
        value_container::ValueContainer,
    },
};
use crate::traits::convert_value_container::ConvertValueContainer;

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
    /// Represents any single value
    SingleValue,
    /// Represents no parts.
    None,
}

/// A trait for types that can be constructed from parts.
pub trait FromParts: HasPartsKind {
    /// Tries to construct the implementing type from the given map parts.
    fn try_from_map_parts(_parts: Map) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Self::try_from_map_parts_with_tag(_parts, None)
    }

    /// Tries to construct the implementing type from the given list parts.
    fn try_from_list_parts(_parts: List) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Self::try_from_list_parts_with_tag(_parts, None)
    }

    /// Tries to construct the implementing type from a single value.
    fn try_from_single_value(_value: ValueContainer) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Err(())
    }

    fn try_from_map_parts_with_tag(
        _parts: Map,
        _tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Err(())
    }

    fn try_from_list_parts_with_tag(
        _parts: List,
        _tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Err(())
    }

    fn try_from_single_value_with_tag(
        _value: ValueContainer,
        _tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Err(())
    }
}

/// Trait for types that have a specific kind of parts.
pub trait HasPartsKind {
    /// Returns the kind of parts that the implementing type can be converted into.
    /// This can be used to check if the conversion is possible before attempting it.
    /// If this returns `PartsKind::None`, then the conversion is not possible.
    /// If this returns `PartsKind::List`, then the conversion is possible and `try_into_list_parts()` can be called.
    /// If this returns `PartsKind::Map`, then the conversion is possible and `try_into_map_parts()` can be called.
    /// If this returns `PartsKind::SingleValue`, then the conversion is possible and `try_from_single_value()` can be called.
    fn parts_kind(&self) -> PartsKind {
        PartsKind::None
    }
}

/// A trait for types that can be converted into parts.
pub trait IntoParts: HasPartsKind {
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

    /// Converts the implementing type into a single value.
    /// Returns an error if the conversion is not possible.
    /// You can check if the conversion is possible by calling `parts_kind()`
    /// and checking if it returns `PartsKind::SingleValue`.
    fn try_into_single_value<'a>(
        self: Box<Self>,
        _cache: &'a mut SharedReferencesCache,
    ) -> Result<ValueContainer, ()>
    where
        Self: 'a,
    {
        Err(())
    }
}
