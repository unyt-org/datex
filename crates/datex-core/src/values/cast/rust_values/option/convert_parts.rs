use crate::{
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    prelude::*,
};

impl<T: IntoParts> IntoParts for Option<T> {
    fn try_into_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Parts<'a>, ()>
    where
        Self: 'a,
    {
        match *self {
            Some(value) => Box::new(value)
                .try_into_parts(cache),
            None => Err(()),
        }
    }

    fn try_as_parts<'a>(
        &'a self,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<BorrowedParts<'a>, ()> {
        match self {
            Some(value) => value.try_as_parts(cache),
            None => Err(()),
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
