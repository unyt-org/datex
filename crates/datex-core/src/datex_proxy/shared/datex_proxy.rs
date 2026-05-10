use crate::{
    datex_proxy::{
        DatexProxy, DatexProxyDeserialize, DatexProxyInfallibleSerialize,
        DatexProxySerialize, shared::Shared,
    },
    values::value_container::ValueContainer,
};

impl<T: DatexProxy> DatexProxyInfallibleSerialize for Shared<T> {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(self.container)
    }
}
impl<T: DatexProxy> DatexProxySerialize for Shared<T> {
    fn try_to_value_container(self) -> Result<ValueContainer, ()> {
        Ok(self.to_value_container())
    }
}
impl<T: DatexProxy> DatexProxyDeserialize for Shared<T> {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, ()> {
        match value {
            ValueContainer::Shared(container) => {
                Shared::try_from(container).map_err(|_| ())
            }
            _x => Err(()),
        }
    }
}

impl<T: DatexProxy> DatexProxy for Shared<T> {}
