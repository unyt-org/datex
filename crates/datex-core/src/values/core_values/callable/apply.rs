use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    traits::{apply::Apply, structural_eq::StructuralEq},
    types::r#type::Type,
    values::{
        core_values::callable::Callable, value_container::ValueContainer,
    },
};
use core::fmt::{Display, Formatter};

impl Apply for Callable {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.call(args)
    }
    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.call(&[arg.clone()])
    }
}
