use crate::{
    traits::apply::{Apply, ApplyError},
    values::{
        core_value::CoreValue, value::Value, value_container::ValueContainer,
    },
};

impl Apply for Value {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self.inner {
            CoreValue::Callable(ref callable) => callable.apply(args),
            _ => Err(ApplyError::UnsupportedApply),
        }
    }
    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self.inner {
            CoreValue::Callable(ref callable) => callable.apply_single(arg),
            _ => Err(ApplyError::UnsupportedApply),
        }
    }
}
