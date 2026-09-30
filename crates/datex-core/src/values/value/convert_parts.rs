use crate::{
    prelude::*,
    preludes::derive::{CoreValue, SharedReferencesCache},
    traits::convert_parts::{FromParts, IntoParts, PartsKind, WithPartsKind},
    values::{
        core_values::{list::List, map::Map},
        value::{
            Value,
            value_classification::{ValueClassification, ValueTag},
        },
    },
};

impl FromParts for Value {
    fn try_from_map_parts_with_tag(
        parts: Map,
        tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        if let Some(tag) = tag {
            Ok(Value::new(
                CoreValue::try_from_map_parts_with_tag(parts, None)?,
                ValueClassification::Tag(ValueTag {
                    tag: tag.to_string(),
                    is_empty: false,
                }),
            ))
        } else {
            Ok(CoreValue::try_from_map_parts_with_tag(parts, tag)?.into())
        }
    }

    fn try_from_list_parts_with_tag(
        parts: List,
        tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(CoreValue::try_from_list_parts_with_tag(parts, tag)?.into())
    }
}

impl WithPartsKind for Value {
    fn parts_kind(&self) -> PartsKind {
        self.inner.parts_kind()
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
        Box::new(self.inner).try_into_map_parts(cache)
    }

    fn try_into_list_parts<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<List, ()>
    where
        Self: 'a,
    {
        Box::new(self.inner).try_into_list_parts(cache)
    }
}
