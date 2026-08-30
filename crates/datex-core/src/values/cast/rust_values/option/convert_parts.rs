use crate::preludes::derive::{SharedReferencesCache};
use crate::traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts};
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedValue};
use crate::values::value::Value;

impl<T: IntoParts> IntoParts for Option<T> {
    fn into_parts(self, cache: &mut SharedReferencesCache) -> Option<Parts> {
        match self {
            Some(value) => value.into_parts(cache),
            None => None,
        }
    }

    fn as_parts(&self, cache: &mut SharedReferencesCache) -> Option<BorrowedParts> {
        match self {
            Some(value) => value.as_parts(cache),
            None => None,
        }
    }
}

impl<T: FromParts> FromParts for Option<T> {

    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized
    {
        Ok(Some(T::try_from_parts(parts)?))
    }
}