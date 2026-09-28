use crate::{
    prelude::*,
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
};
use core::ops::Deref;

impl<T: IntoParts> IntoParts for Box<T> {
    fn try_into_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Parts<'a>, ()>
    where
        Self: 'a,
    {
        let inner = *self;
        inner
            .try_into_parts(cache)
    }

    fn try_as_parts<'a>(
        &'a self,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<BorrowedParts<'a>, ()> {
        self.deref().try_as_parts(cache)
    }
}

impl<T: FromParts> FromParts for Box<T> {
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Box::new(T::try_from_parts(parts)?))
    }
}
