use crate::{
    runtime::execution::ExecutionError, traits::apply::Apply,
    values::value_container::ValueContainer,
};
impl Apply for ValueContainer {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match self {
            ValueContainer::Local(value) => value.apply(args),
            ValueContainer::Shared(reference) => reference.apply(args),
        }
    }

    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match self {
            ValueContainer::Local(value) => value.apply_single(arg),
            ValueContainer::Shared(reference) => reference.apply_single(arg),
        }
    }
}
