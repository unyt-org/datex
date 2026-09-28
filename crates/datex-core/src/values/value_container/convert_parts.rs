use crate::{
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    values::{value::Value, value_container::ValueContainer},
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
    fn try_into_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Parts<'a>, ()>
    where
        Self: 'a,
    {
        match self {
            ValueContainer::Local(value) => Box::new(value).try_into_parts(cache).map_err(|inner| ()),
            ValueContainer::Shared(shared) => Err(()),
        }
    }

    fn try_as_parts<'a>(
        &'a self,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<BorrowedParts<'a>, ()> {
        match self {
            ValueContainer::Local(value) => value.try_as_parts(cache),
            ValueContainer::Shared(_shared) => Err(()),
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
