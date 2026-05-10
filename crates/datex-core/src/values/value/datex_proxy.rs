use crate::{
    datex_proxy::{
        DatexProxy, DatexProxyDeserialize, DatexProxyInfallibleSerialize,
        DatexProxySerialize,
    },
    values::{value::Value, value_container::ValueContainer},
};
use crate::datex_proxy::{TryFromValueContainerError, TryToValueContainerError};

impl DatexProxyInfallibleSerialize for Value {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Local(self)
    }
}
impl DatexProxySerialize for Value {
    fn try_to_value_container(self) -> Result<ValueContainer, TryToValueContainerError> {
        Ok(self.to_value_container())
    }
}
impl DatexProxyDeserialize for Value {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, TryFromValueContainerError> {
        Ok(match value {
            ValueContainer::Local(v) => v,
            _x => return Err(TryFromValueContainerError("Expected ValueContainer::Local, got ValueContainer::Shared".to_string())),
        })
    }
}

impl DatexProxy for Value {}
