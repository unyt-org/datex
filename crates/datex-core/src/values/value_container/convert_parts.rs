use crate::{
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    values::{value::Value, value_container::ValueContainer, core_values::map::Map, core_values::list::List},
    prelude::*,
};
use crate::traits::convert_parts::PartsKind;

impl IntoParts for ValueContainer {
    fn parts_kind(&self) -> PartsKind {
        match self {
            ValueContainer::Local(value) => value.parts_kind(),
            ValueContainer::Shared(_shared) => PartsKind::None,
        }
    }
    fn try_into_map_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()>
    where
        Self: 'a,
    {
        match self {
            ValueContainer::Local(value) => Box::new(value).try_into_map_parts(cache),
            ValueContainer::Shared(shared) => Err(()),
        }
    }

    fn try_into_list_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        match self {
            ValueContainer::Local(value) => Box::new(value).try_into_list_parts(cache),
            ValueContainer::Shared(shared) => Err(()),
        }
    }
}

impl FromParts for ValueContainer {
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(ValueContainer::Local(Value::try_from_parts(parts)?))
    }
}
