use crate::{
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    prelude::*,
    values::core_values::map::Map,
};
use crate::preludes::derive::{List, PartsKind};

impl<T: IntoParts> IntoParts for Option<T> {
    fn parts_kind(&self) -> PartsKind {
        match self {
            Some(value) => value.parts_kind(),
            None => PartsKind::None,
        }
    }
    fn try_into_map_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()>
    where
        Self: 'a,
    {
        match *self {
            Some(value) => Box::new(value)
                .try_into_map_parts(cache),
            None => Err(()),
        }
    }

    fn try_into_list_parts<'a>(
        self: Box<Self>, 
        cache: &'a mut SharedReferencesCache
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        match *self {
            Some(value) => Box::new(value)
                .try_into_list_parts(cache),
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
