use crate::{
    datex_proxy::{
        DatexProxy, DatexProxyDeserialize, DatexProxyInfallibleSerialize,
        DatexProxySerialize,
    },
    values::value_container::ValueContainer,
};

impl DatexProxyInfallibleSerialize for ValueContainer {
    fn to_value_container(self) -> ValueContainer {
        self
    }
}
impl DatexProxySerialize for ValueContainer {
    fn try_to_value_container(self) -> Result<ValueContainer, ()> {
        Ok(self)
    }
}
impl DatexProxyDeserialize for ValueContainer {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, ()> {
        Ok(value)
    }
}

/// Custom [DatexProxy] for [ValueContainer] - just return as is
impl DatexProxy for ValueContainer {}
