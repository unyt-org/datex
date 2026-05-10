use crate::{
    datex_proxy::{
        DatexProxy, DatexProxyDeserialize, DatexProxyInfallibleSerialize,
        DatexProxySerialize,
    },
    shared_values::SharedContainer,
    values::value_container::ValueContainer,
};
use crate::datex_proxy::{TryFromValueContainerError, TryToValueContainerError};

impl DatexProxyInfallibleSerialize for SharedContainer {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(self)
    }
}
impl DatexProxySerialize for SharedContainer {
    fn try_to_value_container(self) -> Result<ValueContainer, TryToValueContainerError> {
        Ok(self.to_value_container())
    }
}
impl DatexProxyDeserialize for SharedContainer {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, TryFromValueContainerError> {
        Ok(match value {
            ValueContainer::Shared(v) => v, 
            _ => return Err(TryFromValueContainerError("Expected ValueContainer::Shared, got ValueContainer::Local".to_string())),
        })
    }
}

impl DatexProxy for SharedContainer {}
