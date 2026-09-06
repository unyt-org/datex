use crate::{
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
};

impl<T: IntoParts> IntoParts for Option<T> {
    fn into_parts<'a>(
        self,
        cache: &'a mut SharedReferencesCache,
    ) -> Option<Parts<'a>>
    where
        Self: 'a,
    {
        match self {
            Some(value) => value.into_parts(cache),
            None => None,
        }
    }

    fn as_parts<'a>(
        &'a self,
        cache: &'a mut SharedReferencesCache,
    ) -> Option<BorrowedParts<'a>> {
        match self {
            Some(value) => value.as_parts(cache),
            None => None,
        }
    }
}

impl<T: FromParts> FromParts for Option<T> {
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Some(T::try_from_parts(parts)?))
    }
}
