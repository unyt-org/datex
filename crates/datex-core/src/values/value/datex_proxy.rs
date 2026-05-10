use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize, DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize,
    },
    values::{value::Value, value_container::ValueContainer},
};
use crate::datex_proxy::{TryFromDatexValueError, TryToDatexValueError};

impl DatexValueContainerProxyInfallibleSerialize for Value {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Local(self)
    }
}
impl DatexValueContainerProxySerialize for Value {
    fn try_to_value_container(self) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.to_value_container())
    }
}
impl DatexValueContainerProxyDeserialize for Value {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, TryFromDatexValueError> {
        Ok(match value {
            ValueContainer::Local(v) => v,
            _x => return Err(TryFromDatexValueError("Expected ValueContainer::Local, got ValueContainer::Shared".to_string())),
        })
    }
}

impl DatexValueContainerProxy for Value {}
