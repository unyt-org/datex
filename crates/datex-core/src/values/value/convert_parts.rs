use crate::{
    prelude::*,
    preludes::derive::{SharedReferencesCache},
    traits::convert_parts::{FromParts, IntoParts, PartsKind, HasPartsKind},
    values::{
        core_values::{list::List, map::Map},
        value::{
            Value,
        },
    },
};
use crate::preludes::derive::ValueContainer;
use crate::values::core_value_with_classification::CoreValueWithClassification;

impl HasPartsKind for Value {
    fn parts_kind(&self) -> PartsKind {
        match self {
            Value::Native(native) => native.value.parts_kind(),
            Value::Core(core) => core.parts_kind(),
        }
    }
}

impl FromParts for Value {
    fn try_from_map_parts_with_tag(
        parts: Map,
        tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Value::core(
            CoreValueWithClassification::try_from_map_parts_with_tag(parts, tag)?,
        ))
    }

    fn try_from_list_parts_with_tag(
        parts: List,
        tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Value::Core(
            CoreValueWithClassification::try_from_list_parts_with_tag(parts, tag)?,
        ))
    }
    
    fn try_from_single_value_with_tag(
        value: ValueContainer,
        tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Value::core(
            CoreValueWithClassification::try_from_single_value_with_tag(value, tag)?,
        ))
    }
}
impl IntoParts for Value {
    fn try_into_map_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()>
    where
        Self: 'a,
    {
        match self {
            Value::Core(core) => Box::new(core).try_into_map_parts(cache),
            Value::Native(native) => native.value.try_into_map_parts(cache),
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
            Value::Core(core) => Box::new(core).try_into_list_parts(cache),
            Value::Native(native) => native.value.try_into_list_parts(cache),
        }
    }

    fn try_into_single_value<'a>(self: Box<Self>, cache: &'a mut SharedReferencesCache) -> Result<ValueContainer, ()>
    where
        Self: 'a,
    {
        match self {
            Value::Core(core) => Box::new(core).try_into_single_value(cache),
            Value::Native(native) => native.value.try_into_single_value(cache),
        }
    }
}
