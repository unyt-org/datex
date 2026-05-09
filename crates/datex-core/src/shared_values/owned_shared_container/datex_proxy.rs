use crate::datex_proxy::{DatexProxy, DatexProxyDeserialize, DatexProxyInfallibleSerialize, DatexProxySerialize};
use crate::shared_values::{OwnedSharedContainer, SharedContainer};
use crate::values::value_container::ValueContainer;

impl DatexProxyInfallibleSerialize for OwnedSharedContainer {
    fn to_value_container(self) -> ValueContainer {
       ValueContainer::Shared(SharedContainer::Owned(self))
    }
}
impl DatexProxySerialize for OwnedSharedContainer {
    fn try_to_value_container(self) -> Result<ValueContainer, ()> {
        Ok(self.to_value_container())
    }
}
impl DatexProxyDeserialize for OwnedSharedContainer {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, ()> {
        Ok(match value {
            ValueContainer::Shared(SharedContainer::Owned(owned)) => owned,
            x => return Err(()),
        })
    }
}

impl DatexProxy for OwnedSharedContainer {}