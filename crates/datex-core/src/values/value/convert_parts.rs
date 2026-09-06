use crate::{
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    values::value::Value,
};

impl FromParts for Value {
    fn try_from_parts(_parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl IntoParts for Value {
    fn into_parts<'a>(
        self,
        _cache: &'a mut SharedReferencesCache,
    ) -> Option<Parts<'a>>
    where
        Self: Sized + 'a,
    {
        todo!()
    }
    fn as_parts(
        &self,
        _cache: &mut SharedReferencesCache,
    ) -> Option<BorrowedParts<'_>> {
        todo!()
    }
}
