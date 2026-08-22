use crate::{
    traits::apply::{Apply, ApplyError},
    values::value_container::ValueContainer,
};
impl Apply for ValueContainer {
    fn try_apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self {
            ValueContainer::Local(value) => value.try_apply(args),
            ValueContainer::Shared(reference) => reference.try_apply(args),
        }
    }

    fn try_apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        match self {
            ValueContainer::Local(value) => value.try_apply_single(arg),
            ValueContainer::Shared(reference) => {
                reference.try_apply_single(arg)
            }
        }
    }
}
