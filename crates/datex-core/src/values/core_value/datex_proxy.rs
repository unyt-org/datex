use crate::datex_proxy::{DatexProxy, DatexProxyDeserialize, DatexProxyInfallibleSerialize, DatexProxySerialize};
use crate::values::core_value::CoreValue;
use crate::values::value_container::ValueContainer;

impl DatexProxyInfallibleSerialize for CoreValue {
    fn to_value_container(self) -> ValueContainer {
       ValueContainer::Local(self.into())
    }
}
impl DatexProxySerialize for CoreValue {
    fn try_to_value_container(self) -> Result<ValueContainer, ()> {
        Ok(self.to_value_container())
    }
}
impl DatexProxyDeserialize for CoreValue {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, ()> {
        Ok(match value {
            ValueContainer::Local(v) => v.inner,
            x => return Err(()),
        })
    }
}

impl DatexProxy for CoreValue {}