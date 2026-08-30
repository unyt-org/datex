use core::ops::Deref;
use crate::preludes::derive::{SharedReferencesCache};
use crate::traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts};

impl<T: IntoParts> IntoParts for Box<T> {
    fn into_parts<'a>(self, cache: &'a mut SharedReferencesCache) -> Option<Parts<'a>> where Self: Sized + 'a {
        let inner = *self;
        inner.into_parts(cache)
    }

    fn as_parts<'a>(&'a self, cache: &'a mut SharedReferencesCache) -> Option<BorrowedParts<'a>> {
        self.deref().as_parts(cache)
    }
}

impl<T: FromParts> FromParts for Box<T> {

    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized
    {
        Ok(Box::new(T::try_from_parts(parts)?))
    }
}