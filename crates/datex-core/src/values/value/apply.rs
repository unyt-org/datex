use crate::{
    prelude::*,
    runtime::Runtime,
    traits::apply::{Apply, ApplyError},
    values::{
        core_value::CoreValue, value::Value, value_container::ValueContainer,
    },
};
impl Apply for Value {
    fn try_apply_sync(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self.inner {
            CoreValue::Callable(ref callable) => {
                callable.try_apply_sync(runtime, args)
            }
            _ => Err(ApplyError::UnsupportedApply),
        }
    }

    async fn try_apply_async(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self.inner {
            CoreValue::Callable(ref callable) => {
                callable.try_apply_async(runtime, args).await
            }
            _ => Err(ApplyError::UnsupportedApply),
        }
    }
}
