pub mod json;
mod rust_core_values;
use crate::values::value_container::ValueContainer;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

pub trait DatexValueProxy: TryFrom<ValueContainer> + From<Self>
where
    ValueContainer: From<Self>,
{
}

/// Datex Proxy that supports inner serde Serialize/Deserialize values
pub trait DatexValueProxyWithSerde:
    TryFrom<ValueContainer> + TryInto<ValueContainer>
{
}

impl<T: Serialize> DatexValueProxyWithSerde for T where
    Self: TryFrom<ValueContainer> + TryInto<ValueContainer>
{
}

// impl<T: Serialize> TryFrom<T> for ValueContainer {
//     type Error = ();

//     fn try_from(value: impl Serialize) -> Result<Self, Self::Error> {
//         let json = serde_json::to_value(value).map_err(|_| ())?;
//         Ok(ValueContainer::from(json))
//     }
// }

/// Convert a serde serializable type to a ValueContainer
/// This is used for user defined types, not annotated with the DATEX macro
pub fn serde_to_value_container<T>(value: T) -> Result<ValueContainer, ()>
where
    T: Serialize,
{
    let json = serde_json::to_value(value).map_err(|_| ())?;
    Ok(ValueContainer::from(json))
}

/// Convert a ValueContainer to a serde deserializable type
/// This is used for user defined types, not annotated with the DATEX macro
pub fn serde_from_value_container<T>(value: ValueContainer) -> Result<T, ()>
where
    T: DeserializeOwned,
{
    let json: serde_json::Value = value.try_into()?;
    serde_json::from_value(json).map_err(|_| ())
}
