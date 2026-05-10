use crate::{
    datex_proxy::{
        DatexProxy, DatexProxyDeserialize, DatexProxyInfallibleSerialize,
        DatexProxySerialize,
    },
    shared_values::{OwnedSharedContainer, SharedContainer},
    values::value_container::ValueContainer,
};
use crate::datex_proxy::{TryFromValueContainerError, TryToValueContainerError};

impl DatexProxyInfallibleSerialize for OwnedSharedContainer {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(SharedContainer::Owned(self))
    }
}
impl DatexProxySerialize for OwnedSharedContainer {
    fn try_to_value_container(self) -> Result<ValueContainer, TryToValueContainerError> {
        Ok(self.to_value_container())
    }
}
impl DatexProxyDeserialize for OwnedSharedContainer {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, TryFromValueContainerError> {
        Ok(match value {
            ValueContainer::Shared(SharedContainer::Owned(owned)) => owned,
            _ => return Err(TryFromValueContainerError("Expected ValueContainer::Shared(SharedContainer::Owned), got something else".to_string())),
        })
    }
}

impl DatexProxy for OwnedSharedContainer {}
