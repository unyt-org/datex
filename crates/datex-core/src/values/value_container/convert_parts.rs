use crate::{
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    values::{value::Value, value_container::ValueContainer},
};

impl IntoParts for ValueContainer {
    fn into_parts<'a>(
        self,
        cache: &'a mut SharedReferencesCache,
    ) -> Option<Parts<'a>>
    where
        Self: Sized + 'a,
    {
        match self {
            ValueContainer::Local(value) => value.into_parts(cache),
            ValueContainer::Shared(_shared) => None,
        }
    }

    fn as_parts<'a>(
        &'a self,
        cache: &'a mut SharedReferencesCache,
    ) -> Option<BorrowedParts<'a>> {
        match self {
            ValueContainer::Local(value) => value.as_parts(cache),
            ValueContainer::Shared(_shared) => None,
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
