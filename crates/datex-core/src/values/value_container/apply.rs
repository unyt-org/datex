use crate::{
    prelude::*,
    runtime::Runtime,
    traits::apply::{Apply, ApplyArgument, ApplyError},
    values::value_container::ValueContainer,
};

impl Apply for ValueContainer {
    fn try_apply_sync(
        &self,
        runtime: &Runtime,
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
        match self {
            ValueContainer::Local(value) => value.try_apply_sync(runtime, args),
            ValueContainer::Shared(reference) => {
                reference.try_apply_sync(runtime, args)
            }
        }
    }

    async fn try_apply_async(
        &self,
        runtime: &Runtime,
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
        match self {
            ValueContainer::Local(value) => {
                value.try_apply_async(runtime, args).await
            }
            ValueContainer::Shared(shared_container) => {
                shared_container.try_apply_async(runtime, args).await
            }
        }
    }
}
