use crate::{
    prelude::*,
    preludes::derive::{PartsKind, SharedReferencesCache},
    traits::convert_parts::{FromParts, IntoParts, WithPartsKind},
    values::core_values::{list::List, map::Map},
};
use core::ops::Deref;

impl<T: WithPartsKind> WithPartsKind for Box<T> {
    fn parts_kind(&self) -> PartsKind {
        self.deref().parts_kind()
    }
}

impl<T: IntoParts> IntoParts for Box<T> {
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

    fn try_into_list_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        let inner = *self;
        inner.try_into_list_parts(cache)
    }
}

impl<T: FromParts> FromParts for Box<T> {
    fn try_from_map_parts_with_tag(
        _parts: Map,
        _tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Box::new(T::try_from_map_parts_with_tag(_parts, _tag)?))
    }

    fn try_from_list_parts_with_tag(
        parts: List,
        tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Box::new(T::try_from_list_parts_with_tag(parts, tag)?))
    }
}
