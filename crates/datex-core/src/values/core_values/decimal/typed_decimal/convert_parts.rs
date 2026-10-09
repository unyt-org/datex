use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::{
        convert_parts::{FromParts, HasPartsKind, IntoParts},
        convert_value_container::ConvertValueContainer,
    },
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

/// Default implementations - cannot be split into parts
impl IntoParts for TypedDecimal {
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
impl FromParts for TypedDecimal {
    fn try_from_single_value_with_tag(
        value: ValueContainer,
        _tag: Option<&str>,
    ) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Self::try_from_value_container(value).map_err(|_| ())
    }
}
impl HasPartsKind for TypedDecimal {}
