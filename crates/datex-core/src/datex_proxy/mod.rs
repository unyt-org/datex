pub mod serde_mapping;

use crate::values::value_container::ValueContainer;
use serde::{Serialize, de::DeserializeOwned};
use crate::values::value::Value;

/// Base DATEX trait for value proxy. Must implement [DatexProxyDeserialize] and [DatexProxySerialize]
pub trait DatexProxy: Sized + DatexProxyDeserialize + DatexProxySerialize {

}

/// Deserialization from a [ValueContainer] to a rust value
pub trait DatexProxyDeserialize: Sized {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, ()>;
}

/// Serialization from a [ValueContainer] to a rust value. Might fail if serde values are serialized.
pub trait DatexProxySerialize {
    fn try_to_value_container(self) -> Result<ValueContainer, ()>;
}

/// Infallible serialization from a [ValueContainer] to a rust value. 
/// Only works if no serde values are serialized.
pub trait DatexProxyInfallibleSerialize {
    fn to_value_container(self) -> ValueContainer;
}

/// Default [DatexProxy] implementation for all types that implement Serialize and DeserializeOwned
impl<T> DatexProxySerialize for T
where
    T: Serialize,
{
    /// Converts a [Serialize] value into a [ValueContainer] by first converting it to a [serde_value::Value] and then deserializing it into a [ValueContainer].
    default fn try_to_value_container(self) -> Result<ValueContainer, ()> {
        let serde_val = serde_value::to_value(self).map_err(|_| ())?;
        serde_val.deserialize_into().map_err(|_| ())
    }
}

impl<T> DatexProxyDeserialize for T
where
    T: DeserializeOwned,
{
    /// Converts a [ValueContainer] into a [DeserializeOwned] type by first converting it to a [serde_value::Value] and then deserializing it into the target type.
    default fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, ()> {
        let serde_val = serde_value::to_value(value).map_err(|_| ())?;
        T::deserialize(serde_val).map_err(|_| ())
    }
}