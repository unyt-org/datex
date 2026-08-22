use crate::{
    traits::apply::{Apply, ApplyError},
    values::{
        core_value::CoreValue, value::Value, value_container::ValueContainer,
    },
};

impl Apply for Value {
    fn try_apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self.inner {
            CoreValue::Callable(ref callable) => callable.try_apply(args),
            _ => Err(ApplyError::UnsupportedApply),
        }
    }
    fn try_apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self.inner {
            CoreValue::Callable(ref callable) => callable.try_apply_single(arg),
            _ => Err(ApplyError::UnsupportedApply),
        }
    }
}
