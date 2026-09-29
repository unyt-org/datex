use crate::{
    prelude::*,
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    values::core_values::map::Map,
};
use crate::traits::convert_parts::PartsKind;

impl IntoParts for Map {
    fn parts_kind(&self) -> PartsKind {
        PartsKind::Map
    }

    fn try_into_map_parts<'a>(
        self: Box<Self>,
        _cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()>
    where
        Self: 'a,
    {
        Ok(*self)
    }
}

impl FromParts for Map {
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
    {
        match parts {
            Parts::Map(iter) => {
                let mut map = Map::default();
                for (key, value) in iter {
                    map.set_unchecked(key, value);
                }
                Ok(map)
            }
            _ => Err(()),
        }
    }
}
