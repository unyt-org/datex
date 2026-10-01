use crate::{
    traits::convert_parts::{FromParts, IntoParts, HasPartsKind},
    values::core_values::range::Range,
    prelude::*,
};
use crate::preludes::derive::{SharedReferencesCache, ValueContainer};
use crate::traits::convert_value_container::ConvertValueContainer;

impl FromParts for Range {
    fn try_from_single_value_with_tag(value: ValueContainer, _tag: Option<&str>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Self::try_from_value_container(value).map_err(|_| ())
    }
}
impl IntoParts for Range {
    fn try_into_single_value<'a>(
        self: Box<Self>,
        cache: &'a mut SharedReferencesCache,
    ) -> Result<ValueContainer, ()>
    where
        Self: 'a,
    {
        Ok(self.to_value_container(cache))
    }
}
impl HasPartsKind for Range {}
