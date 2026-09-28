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
    fn try_from_parts(parts: Parts) -> Result<Self, ()>
    where
        Self: Sized,
    {
        match parts {
            Parts::Map(iter) => {
                Ok(Value::from(Map::from(iter.collect_vec())))
            }
            Parts::List(iter) => {
                Ok(Value::from(List::new(iter.collect_vec())))
            }
        }
    }
}

impl IntoParts for Value {
    fn parts_kind(&self) -> PartsKind {
        match &self.inner {
            CoreValue::Map(_) => PartsKind::Map,
            CoreValue::List(_) => PartsKind::List,
            CoreValue::Native(native) => native.value.parts_kind(),
            _ => PartsKind::None,
        }
    }

    fn try_into_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<Parts<'a>, ()> where Self: 'a
    {
        match self.inner {
            CoreValue::Map(map) => Box::new(map).try_into_parts(cache),
            CoreValue::List(list) => Box::new(list).try_into_parts(cache),
            CoreValue::Native(native) => native.value.try_into_parts(cache),
            _ => Err(()),
        }
    }
    fn try_as_parts<'a>(
        &'a self,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<BorrowedParts<'a>, ()> {
        match &self.inner {
            CoreValue::Map(map) => map.try_as_parts(cache),
            CoreValue::List(list) => list.try_as_parts(cache),
            CoreValue::Native(native) => native.value.try_as_parts(cache),
            _ => Err(()),
        }
    }
}
