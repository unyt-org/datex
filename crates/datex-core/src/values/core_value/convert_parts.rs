use crate::{
    prelude::*,
    preludes::derive::{CoreValue, SharedReferencesCache},
    traits::convert_parts::{
        FromParts, IntoParts, Parts, PartsKind, HasPartsKind,
    },
    values::core_values::{list::List, map::Map},
};
use crate::preludes::derive::{ValueClassification, ValueContainer};
use crate::values::value::Value;

impl FromParts for CoreValue {
    fn try_from_map_parts_with_tag(parts: Map, _tag: Option<&str>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(CoreValue::Map(parts))
    }

    fn try_from_list_parts_with_tag(parts: List, _tag: Option<&str>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(CoreValue::List(parts))
    }

    fn try_from_single_value_with_tag(value: ValueContainer, _tag: Option<&str>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        match value {
            ValueContainer::Local(Value::Core(value)) => Ok(value.inner),
            _ => Err(())
        }
    }
}

impl HasPartsKind for CoreValue {
    fn parts_kind(&self) -> PartsKind {
        match self {
            CoreValue::Map(_) => PartsKind::Map,
            CoreValue::List(_) => PartsKind::List,
            _ => PartsKind::SingleValue,
        }
    }
}

impl IntoParts for CoreValue {
    fn try_into_map_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()>
    where
        Self: 'a,
    {
        match self {
            CoreValue::Map(map) => Ok(map),
            _ => Err(()),
        }
    }

    fn try_into_list_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        match self {
            CoreValue::List(list) => Ok(list),
            _ => Err(()),
        }
    }

    fn try_into_single_value<'a>(self: Box<Self>, cache: &'a mut SharedReferencesCache) -> Result<ValueContainer, ()>
    where
        Self: 'a,
    {
        match self {
            _ => {
                Ok(ValueContainer::Local(Value::new(*self)))
            }
        }
    }
}
