use crate::preludes::derive::SharedReferencesCache;
use crate::traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts};
use crate::values::value::Value;

impl FromParts for Value {
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl IntoParts for Value {
    fn into_parts<'a>(self, cache: &'a mut SharedReferencesCache) -> Option<Parts<'a>>
    where
        Self: Sized + 'a,
    {
        todo!()
    }
    fn as_parts(&self, cache: &mut SharedReferencesCache) -> Option<BorrowedParts> {
        todo!()
    }
}