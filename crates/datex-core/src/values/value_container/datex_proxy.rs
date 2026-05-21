use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError,
    },
    values::value_container::ValueContainer,
};

impl DatexValueContainerProxyInfallibleSerialize for ValueContainer {
    fn to_value_container(self) -> ValueContainer {
        self
    }
}
impl DatexValueContainerProxySerialize for ValueContainer {
    fn try_to_value_container(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self)
    }
}
impl DatexValueContainerProxyDeserialize for ValueContainer {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        Ok(value)
    }
}

/// Custom [DatexValueContainerProxy] for [ValueContainer] - just return as is
impl DatexValueContainerProxy for ValueContainer {}
