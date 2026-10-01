use crate::{
    prelude::*,
    preludes::derive::{List, PartsKind, SharedReferencesCache},
    traits::convert_parts::{FromParts, IntoParts, Parts, HasPartsKind},
    values::core_values::map::Map,
};
use crate::preludes::derive::ValueContainer;
use crate::values::value::Value;

impl<T: HasPartsKind> HasPartsKind for Option<T> {
    fn parts_kind(&self) -> PartsKind {
        match self {
            Some(value) => value.parts_kind(),
            None => PartsKind::None,
        }
    }
}
impl<T: IntoParts> IntoParts for Option<T> {
    fn try_into_map_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Map, ()>
    where
        Self: 'a,
    {
        match *self {
            Some(value) => Box::new(value).try_into_map_parts(cache),
            None => Err(()),
        }
    }

    fn try_into_list_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        match *self {
            Some(value) => Box::new(value).try_into_list_parts(cache),
            None => Err(()),
        }
    }

    fn try_into_single_value<'a>(self: Box<Self>, _cache: &'a mut SharedReferencesCache) -> Result<ValueContainer, ()>
    where
        Self: 'a,
    {
        match *self {
            Some(value) => Box::new(value).try_into_single_value(_cache),
            None => Ok(ValueContainer::Local(Value::null()))
        }
    }
}

impl<T: FromParts> FromParts for Option<T> {
    fn try_from_map_parts_with_tag(
        parts: Map,
        tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Some(T::try_from_map_parts_with_tag(parts, tag)?))
    }

    fn try_from_list_parts_with_tag(
        parts: List,
        tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Some(T::try_from_list_parts_with_tag(parts, tag)?))
    }

    fn try_from_single_value_with_tag(value: ValueContainer, tag: Option<&str>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Some(T::try_from_single_value_with_tag(value, tag)?))
    }
}
