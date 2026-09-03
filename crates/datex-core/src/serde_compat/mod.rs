use crate::{
    prelude::*,
    values::{value::Value, value_container::ValueContainer},
};
use serde::{Deserialize, Serialize};

mod serde_mapping;


/// Converts a [Serialize] value into a [ValueContainer] by first converting it to a [serde_val::Value] and then deserializing it into a [ValueContainer].
pub fn try_serde_to_value_container<T: Serialize>(
    value: T,
) -> Result<ValueContainer, ()> {
    let serde_val = serde_val::to_value(value)
        .map_err(|err| ())?;
    serde_val
        .deserialize_into()
        .map_err(|err| ())
}

/// Converts a [ValueContainer] into a rust value by first converting it to a [serde_val::Value] and then deserializing it into the target type.
pub fn try_serde_from_value_container<'de, T: Deserialize<'de>>(
    value: ValueContainer,
) -> Result<T, ()> {
    let serde_val = serde_val::to_value(value)
        .map_err(|err| ())?;
    T::deserialize(serde_val)
        .map_err(|err| ())
}

/// Converts a [Serialize] value into a [Value] by first converting it to a [serde_val::Value] and then deserializing it into a [Value].
pub fn try_serde_to_value<T: Serialize>(
    value: T,
) -> Result<Value, ()> {
    let serde_val = serde_val::to_value(value)
        .map_err(|err| ())?;
    serde_val
        .deserialize_into()
        .map_err(|err| ())
}

/// Converts a [Value] into a rust value by first converting it to a [serde_val::Value] and then deserializing it into the target type.
pub fn try_serde_from_value<'de, T: Deserialize<'de>>(
    value: Value,
) -> Result<T, ()> {
    let serde_val = serde_val::to_value(value)
        .map_err(|err| ())?;
    T::deserialize(serde_val)
        .map_err(|err| ())
}
