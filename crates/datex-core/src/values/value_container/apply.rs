use crate::{
    prelude::*,
    runtime::Runtime,
    traits::apply::{Apply, ApplyError},
    values::value_container::ValueContainer,
};
impl Apply for ValueContainer {
    fn try_apply(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self {
            ValueContainer::Local(value) => value.try_apply(runtime, args),
            ValueContainer::Shared(reference) => {
                reference.try_apply(runtime, args)
            }
        }
    }

    fn try_apply_single(
        &self,
        runtime: &Runtime,
        arg: ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self {
            ValueContainer::Local(value) => {
                value.try_apply_single(runtime, arg)
            }
            ValueContainer::Shared(reference) => {
                reference.try_apply_single(runtime, arg)
            }
        }
    }
}
