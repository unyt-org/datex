use crate::preludes::derive::SharedReferencesCache;
use crate::traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts};
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

impl IntoParts for ValueContainer {
    fn into_parts<'a>(self, cache: &'a mut SharedReferencesCache) -> Option<Parts<'a>>   
    where
        Self: Sized + 'a, 
    {
        match self {
            ValueContainer::Local(value) => value.into_parts(cache),
            ValueContainer::Shared(shared) => None
        }
    }

    fn as_parts<'a>(&'a self, cache: &'a mut SharedReferencesCache) -> Option<BorrowedParts<'a>> {
        match self {
            ValueContainer::Local(value) => value.as_parts(cache),
            ValueContainer::Shared(shared) => None
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