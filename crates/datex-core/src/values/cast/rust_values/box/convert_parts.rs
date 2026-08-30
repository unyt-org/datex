use core::ops::Deref;
use crate::preludes::derive::{SharedReferencesCache};
use crate::traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts};

impl<T: IntoParts> IntoParts for Box<T> {
    fn into_parts(self, cache: &mut SharedReferencesCache) -> Option<Parts> where Self: Sized {
        let inner = *self;
        inner.into_parts(cache)
    }

    fn as_parts(&self, cache: &mut SharedReferencesCache) -> Option<BorrowedParts> {
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