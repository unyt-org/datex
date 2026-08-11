use crate::{
    prelude::*,
    runtime::Runtime,
    traits::apply::{Apply, ApplyError},
    values::{
        core_value::CoreValue, value::Value, value_container::ValueContainer,
    },
};
impl Apply for Value {
    fn try_apply(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self.inner {
            CoreValue::Callable(ref callable) => {
                callable.try_apply(runtime, args)
            }
            _ => Err(ApplyError::UnsupportedApply),
        }
    }
    fn try_apply_single(
        &self,
        runtime: &Runtime,
        arg: ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self.inner {
            CoreValue::Callable(ref callable) => {
                callable.try_apply_single(runtime, arg)
            }
            _ => Err(ApplyError::UnsupportedApply),
        }
    }
}
