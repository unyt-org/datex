use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize, DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize,
    },
    shared_values::{ReferencedSharedContainer, SharedContainer},
    values::value_container::ValueContainer,
};
use crate::datex_proxy::{TryFromDatexValueError, TryToDatexValueError};
use crate::prelude::*;

impl DatexValueContainerProxyInfallibleSerialize for ReferencedSharedContainer {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(SharedContainer::Referenced(self))
    }
}
impl DatexValueContainerProxySerialize for ReferencedSharedContainer {
    fn try_to_value_container(self) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.to_value_container())
    }
}
impl DatexValueContainerProxyDeserialize for ReferencedSharedContainer {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, TryFromDatexValueError> {
        Ok(match value {
            ValueContainer::Shared(SharedContainer::Referenced(reference)) => {
                reference
            }
            _ => return Err(TryFromDatexValueError("Expected ValueContainer::Shared(SharedContainer::Referenced), got something else".to_string())),
        })
    }
}

impl DatexValueContainerProxy for ReferencedSharedContainer {}
