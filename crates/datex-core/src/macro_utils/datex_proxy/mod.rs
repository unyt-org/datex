mod rust_core_values;

use serde::Serialize;
use serde_json::Value;
use crate::values::value_container::ValueContainer;


pub trait DatexValueProxy: TryFrom<ValueContainer> + From<Self>
where ValueContainer: From<Self>,
{

}

/// Datex Proxy that supports inner serde Serialize/Deserialize values
pub trait DatexValueProxyWithSerde: TryFrom<ValueContainer> + TryInto<ValueContainer> {

}


impl<T: Serialize> DatexValueProxyWithSerde for T where Self: TryFrom<ValueContainer> + TryInto<ValueContainer> {

}

impl<T: Serialize> TryFrom<T> for ValueContainer {
    type Error = ();

    fn try_from(value: impl Serialize) -> Result<Self, Self::Error> {
        let json = serde_json::to_value(value).map_err(|_| ())?;
        Ok(ValueContainer::from(json))
    }
}

impl From<Value> for ValueContainer {
    fn from(value: Value) -> Self {
        todo!()
    }
}