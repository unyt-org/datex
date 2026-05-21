use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError,
    },
    prelude::*,
    shared_values::{OwnedSharedContainer, SharedContainer},
    values::value_container::ValueContainer,
};

impl DatexValueContainerProxyInfallibleSerialize for OwnedSharedContainer {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(SharedContainer::Owned(self))
    }
}
impl DatexValueContainerProxySerialize for OwnedSharedContainer {
    fn try_to_value_container(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.to_value_container())
    }
}
impl DatexValueContainerProxyDeserialize for OwnedSharedContainer {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        Ok(match value {
            ValueContainer::Shared(SharedContainer::Owned(owned)) => owned,
            _ => return Err(TryFromDatexValueError("Expected ValueContainer::Shared(SharedContainer::Owned), got something else".to_string())),
        })
    }
}

impl DatexValueContainerProxy for OwnedSharedContainer {}
