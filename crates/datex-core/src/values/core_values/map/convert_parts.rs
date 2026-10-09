use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::convert_parts::{FromParts, IntoParts, PartsKind, HasPartsKind},
    values::core_values::map::Map,
};

impl HasPartsKind for Map {
    fn parts_kind(&self) -> PartsKind {
        PartsKind::Map
    }
}

impl IntoParts for Map {
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
    fn try_from_map_parts_with_tag(parts: Map, _tag: Option<&str>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(parts)
    }
}
