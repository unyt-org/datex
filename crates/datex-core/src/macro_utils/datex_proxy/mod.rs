pub mod serde_mapping;

use crate::values::value_container::ValueContainer;
use serde::{Serialize, de::DeserializeOwned};


pub trait DatexProxy: Sized {
    fn datex_to_value_container(self) -> Result<ValueContainer, ()>;

    fn datex_from_value_container(value: ValueContainer) -> Result<Self, ()>;
}

/// Fallback implementation for all types that implement Serialize and DeserializeOwned
impl<T> DatexProxy for T
where
    T: Serialize + DeserializeOwned,
{
    /// Converts a [Serialize] value into a [ValueContainer] by first converting it to a [serde_value::Value] and then deserializing it into a [ValueContainer].
    default fn datex_to_value_container(self) -> Result<ValueContainer, ()> {
        let serde_val = serde_value::to_value(self).map_err(|_| ())?;
        serde_val.deserialize_into().map_err(|_| ())
    }

    /// Converts a [ValueContainer] into a [DeserializeOwned] type by first converting it to a [serde_value::Value] and then deserializing it into the target type.
    default fn datex_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, ()> {
        let serde_val = serde_value::to_value(value).map_err(|_| ())?;
        T::deserialize(serde_val).map_err(|_| ())
    }
}