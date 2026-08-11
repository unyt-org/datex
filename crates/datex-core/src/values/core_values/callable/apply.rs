use crate::{
    prelude::*,
    runtime::Runtime,
    traits::apply::{Apply, ApplyError},
    values::{
        core_values::callable::Callable, value_container::ValueContainer,
    },
};

impl Apply for Callable {
    fn try_apply(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        Ok(self.call(runtime, args)?)
    }
    fn try_apply_single(
        &self,
        runtime: &Runtime,
        arg: ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        Ok(self.call(runtime, vec![arg.clone()])?)
    }
}
