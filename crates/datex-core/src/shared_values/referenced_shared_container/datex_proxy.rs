use crate::{
    datex_proxy::{
        DatexProxy, DatexProxyDeserialize, DatexProxyInfallibleSerialize,
        DatexProxySerialize,
    },
    shared_values::{ReferencedSharedContainer, SharedContainer},
    values::value_container::ValueContainer,
};
use crate::datex_proxy::{TryFromValueContainerError, TryToValueContainerError};

impl DatexProxyInfallibleSerialize for ReferencedSharedContainer {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(SharedContainer::Referenced(self))
    }
}
impl DatexProxySerialize for ReferencedSharedContainer {
    fn try_to_value_container(self) -> Result<ValueContainer, TryToValueContainerError> {
        Ok(self.to_value_container())
    }
}
impl DatexProxyDeserialize for ReferencedSharedContainer {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, TryFromValueContainerError> {
        Ok(match value {
            ValueContainer::Shared(SharedContainer::Referenced(reference)) => {
                reference
            }
            _ => return Err(TryFromValueContainerError("Expected ValueContainer::Shared(SharedContainer::Referenced), got something else".to_string())),
        })
    }
}

impl DatexProxy for ReferencedSharedContainer {}
