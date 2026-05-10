use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize, DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize,
    },
    values::{core_value::CoreValue, value_container::ValueContainer},
};
use crate::datex_proxy::{TryFromDatexValueError, TryToDatexValueError};

impl DatexValueContainerProxyInfallibleSerialize for CoreValue {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Local(self.into())
    }
}
impl DatexValueContainerProxySerialize for CoreValue {
    fn try_to_value_container(self) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.to_value_container())
    }
}
impl DatexValueContainerProxyDeserialize for CoreValue {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, TryFromDatexValueError> {
        Ok(match value {
            ValueContainer::Local(v) => v.inner,
            _ => return Err(TryFromDatexValueError("Expected ValueContainer::Local, got ValueContainer::Shared".to_string())),
        })
    }
}

impl DatexValueContainerProxy for CoreValue {}
