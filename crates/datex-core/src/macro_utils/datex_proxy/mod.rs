pub mod json;
use crate::values::value_container::ValueContainer;
use serde::{Serialize, de::DeserializeOwned};

/// Native DATEX mapping (Rust primitive types, lists and maps)
pub trait DatexDirect: Serialize + DeserializeOwned + Sized {
    fn datex_direct_to_value_container(self) -> Result<ValueContainer, ()>;

    fn datex_direct_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, ()>;
}

pub trait DatexField: Sized {
    fn datex_to_value_container(self) -> Result<ValueContainer, ()>;

    fn datex_from_value_container(value: ValueContainer) -> Result<Self, ()>;
}

/// Fallback implementation for all types that implement Serialize and DeserializeOwned
impl<T> DatexField for T
where
    T: Serialize + DeserializeOwned,
{
    default fn datex_to_value_container(self) -> Result<ValueContainer, ()> {
        serde_to_value_container(self)
    }

    default fn datex_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, ()> {
        serde_from_value_container(value)
    }
}

/// DatexDirect types bypass serde even though they also implement serde.
impl<T> DatexField for T
where
    T: DatexDirect,
{
    fn datex_to_value_container(self) -> Result<ValueContainer, ()> {
        self.datex_direct_to_value_container()
    }

    fn datex_from_value_container(value: ValueContainer) -> Result<Self, ()> {
        T::datex_direct_from_value_container(value)
    }
}

/// Convert a serde serializable type to a ValueContainer
/// This is used for user defined types, not annotated with the DATEX macro
pub fn serde_to_value_container<T>(value: T) -> Result<ValueContainer, ()>
where
    T: Serialize,
{
    let json = serde_json::to_value(value).map_err(|_| ())?;
    println!("Serialized to JSON: {}", json);
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
