use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize, DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, shared::Shared,
    },
    values::value_container::ValueContainer,
};
use crate::datex_proxy::{TryFromDatexValueError, TryToDatexValueError};
use crate::prelude::*;

impl<T: DatexValueContainerProxy> DatexValueContainerProxyInfallibleSerialize for Shared<T> {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(self.container)
    }
}
impl<T: DatexValueContainerProxy> DatexValueContainerProxySerialize for Shared<T> {
    fn try_to_value_container(self) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.to_value_container())
    }
}
impl<T: DatexValueContainerProxy> DatexValueContainerProxyDeserialize for Shared<T> {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, TryFromDatexValueError> {
        match value {
            ValueContainer::Shared(container) => {
                Shared::try_from(container)
            }
            _ => Err(TryFromDatexValueError("Expected ValueContainer::Shared, ValueContainer::Local".to_string())),
        }
    }
}

impl<T: DatexValueContainerProxy> DatexValueContainerProxy for Shared<T> {}
