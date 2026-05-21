use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError,
    },
    prelude::*,
    shared_values::SharedContainer,
    values::value_container::ValueContainer,
};

impl DatexValueContainerProxyInfallibleSerialize for SharedContainer {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(self)
    }
}
impl DatexValueContainerProxySerialize for SharedContainer {
    fn try_to_value_container(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.to_value_container())
    }
}
impl DatexValueContainerProxyDeserialize for SharedContainer {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        Ok(match value {
            ValueContainer::Shared(v) => v,
            _ => return Err(TryFromDatexValueError(
                "Expected ValueContainer::Shared, got ValueContainer::Local"
                    .to_string(),
            )),
        })
    }
}

impl DatexValueContainerProxy for SharedContainer {}
