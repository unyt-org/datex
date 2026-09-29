use itertools::Itertools;
use crate::{
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{BorrowedParts, FromParts, IntoParts, Parts},
    values::value::Value,
    prelude::*,
};
use crate::preludes::derive::CoreValue;
use crate::traits::convert_parts::PartsKind;
use crate::values::core_values::map::Map;
use crate::values::core_values::list::List;

impl FromParts for Value {
    fn try_from_map_parts(parts: Map) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(CoreValue::try_from_map_parts(parts)?.into())
    }

    fn try_from_list_parts(parts: List) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(CoreValue::try_from_list_parts(parts)?.into())
    }
}

impl IntoParts for Value {
    fn parts_kind(&self) -> PartsKind {
        self.inner.parts_kind()
    }

    fn try_into_map_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()>
    where
        Self: 'a,
    {
        Box::new(self.inner).try_into_map_parts(cache)
    }

    fn try_into_list_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        Box::new(self.inner).try_into_list_parts(cache)
    }
}
