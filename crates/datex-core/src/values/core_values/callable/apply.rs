use crate::{
    traits::apply::{Apply, ApplyError},
    values::{
        core_values::callable::Callable, value_container::ValueContainer,
    },
};
use crate::runtime::Runtime;

impl Apply for Callable {
    fn try_apply(
        &self,
        runtime: &Runtime,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ApplyError> {
        Ok(self.call(runtime, args)?)
    }
    fn try_apply_single(
        &self,
        runtime: &Runtime,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        Ok(self.call(runtime, core::slice::from_ref(arg))?)
    }
}
