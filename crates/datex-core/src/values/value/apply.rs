use crate::{
    prelude::*,
    runtime::Runtime,
    traits::apply::{Apply, ApplyError},
    values::{
        core_value::CoreValue, value::Value, value_container::ValueContainer,
    },
};
use crate::traits::apply::ApplyArgument;

impl Apply for Value {
    fn try_apply_sync(
        &self,
        runtime: &Runtime,
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
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
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
        match self.inner {
            CoreValue::Callable(ref callable) => {
                callable.try_apply_async(runtime, args).await
            }
            _ => Err(ApplyError::UnsupportedApply),
        }
    }
}
