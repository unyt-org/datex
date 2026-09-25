use crate::{
    prelude::*,
    preludes::derive::{DatexNative, StaticClassification, ValueContainer},
    traits::{
        get_datex_type::GetDatexType,
        local_child_path_resolver::LocalChildPathResolver,
    },
    value_updates::errors::UpdateError,
    values::{
        core_values::native::DatexNativeOps, value::Value,
        value_container::value_key::ValueKey,
    },
};
use core::any::Any;

impl<T: DatexNative + GetDatexType + StaticClassification> DatexNative
    for Box<T>
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl<T: DatexNative + GetDatexType + StaticClassification> DatexNativeOps
    for Box<T>
{
}

impl<T: LocalChildPathResolver> LocalChildPathResolver for Box<T> {
    fn resolve_child(
        &mut self,
        key: &ValueKey,
    ) -> Result<&mut ValueContainer, UpdateError> {
        self.as_mut().resolve_child(key)
    }

    fn resolve_value_for_path(
        &mut self,
        first: &ValueKey,
        remaining_path: &[ValueKey],
    ) -> Result<&mut Value, UpdateError> {
        self.as_mut().resolve_value_for_path(first, remaining_path)
    }
}
