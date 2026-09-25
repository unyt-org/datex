use crate::{
    prelude::*,
    preludes::derive::DatexNative,
    traits::local_child_path_resolver::LocalChildPathResolver,
    value_updates::errors::UpdateError,
    values::{
        core_values::native::{DatexNativeBase, DatexNativeOps},
        value::Value,
        value_container::value_key::ValueKey,
    },
};
use core::any::Any;

impl<T: DatexNativeBase + 'static> DatexNative for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl<T: DatexNativeBase + 'static> DatexNativeOps for Vec<T> {}
impl<T: DatexNativeBase + 'static> LocalChildPathResolver for Vec<T> {
    fn resolve_value_for_path(
        &mut self,
        first: &ValueKey,
        remaining_path: &[ValueKey],
    ) -> Result<&mut Value, UpdateError> {
        todo!()
    }
}
