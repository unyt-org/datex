use itertools::Itertools;
use crate::{
    preludes::derive::SharedReferencesCache,
    traits::convert_parts::{FromParts, IntoParts, Parts},
    prelude::*,
};
use crate::preludes::derive::CoreValue;
use crate::traits::convert_parts::PartsKind;
use crate::values::core_values::map::Map;
use crate::values::core_values::list::List;

impl FromParts for CoreValue {
    fn try_from_map_parts(parts: Map) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(CoreValue::Map(parts))
    }
}

impl IntoParts for CoreValue {
    fn parts_kind(&self) -> PartsKind {
        match self {
            CoreValue::Map(_) => PartsKind::Map,
            CoreValue::List(_) => PartsKind::List,
            CoreValue::Native(native) => native.value.parts_kind(),
            _ => PartsKind::None,
        }
    }

    fn try_into_map_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()> where Self: 'a
    {
        match self {
            CoreValue::Map(map) => Ok(map),
            CoreValue::Native(native) => native.value.try_into_map_parts(cache),
            _ => Err(()),
        }
    }

    fn try_into_list_parts<'a>(
        self: Box<Self>, 
        cache: &'a mut SharedReferencesCache
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        match self {
            CoreValue::List(list) => Ok(list),
            CoreValue::Native(native) => native.value.try_into_list_parts(cache),
            _ => Err(()),
        }
    }
}
