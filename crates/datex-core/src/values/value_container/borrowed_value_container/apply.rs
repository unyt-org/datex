use crate::{
    prelude::*,
    runtime::Runtime,
    traits::apply::{Apply, ApplyArgument, ApplyError},
    values::{
        borrowed_value_container::BorrowedValueContainer,
        value_container::ValueContainer,
    },
};

impl<'a> Apply for BorrowedValueContainer<'a> {
    fn try_apply_sync(
        &self,
        runtime: &Runtime,
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
        match self {
            BorrowedValueContainer::Local(value) => {
                value.try_apply_sync(runtime, args)
            }
            BorrowedValueContainer::Shared(reference) => {
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
            BorrowedValueContainer::Local(value) => {
                value.try_apply_async(runtime, args).await
            }
            BorrowedValueContainer::Shared(shared_container) => {
                shared_container.try_apply_async(runtime, args).await
            }
        }
    }
}
