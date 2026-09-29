use crate::{
    prelude::*,
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    values::core_values::map::Map,
    values::core_values::list::List,
};
use core::ops::Deref;
use crate::preludes::derive::{PartsKind};

impl<T: IntoParts> IntoParts for Box<T> {
    fn parts_kind(&self) -> PartsKind {
        self.deref().parts_kind()
    }
    fn try_into_map_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()>
    where
        Self: 'a,
    {
        let inner = *self;
        inner.try_into_map_parts(cache)
    }

    fn try_into_list_parts<'a>(self: Box<Self>, _cache: &'a mut SharedReferencesCache) -> Result<List, ()>
    where
        Self: 'a,
    {
        let inner = *self;
        inner.try_into_list_parts(_cache)
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
