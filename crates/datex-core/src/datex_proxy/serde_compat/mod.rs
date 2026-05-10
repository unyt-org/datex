use serde::{Deserialize, Serialize};
use crate::datex_proxy::{TryFromDatexValueError, TryToDatexValueError};
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

mod serde_mapping;

/// Converts a [Serialize] value into a [ValueContainer] by first converting it to a [serde_value::Value] and then deserializing it into a [ValueContainer].
pub fn try_serde_to_value_container<T: Serialize>(value: T) -> Result<ValueContainer, TryToDatexValueError> {
    let serde_val = serde_value::to_value(value).map_err(|err| TryToDatexValueError(err.to_string()))?;
    serde_val.deserialize_into().map_err(|err| TryToDatexValueError(err.to_string()))
}

/// Converts a [ValueContainer] into a rust value by first converting it to a [serde_value::Value] and then deserializing it into the target type.
pub fn try_serde_from_value_container<'de, T: Deserialize<'de>>(
    value: ValueContainer,
) -> Result<T, TryFromDatexValueError> {
    let serde_val = serde_value::to_value(value).map_err(|err| TryFromDatexValueError(err.to_string()))?;
    T::deserialize(serde_val).map_err(|err| TryFromDatexValueError(err.to_string()))
}

/// Converts a [Serialize] value into a [Value] by first converting it to a [serde_value::Value] and then deserializing it into a [Value].
pub fn try_serde_to_value<T: Serialize>(value: T) -> Result<Value, TryToDatexValueError> {
    let serde_val = serde_value::to_value(value).map_err(|err| TryToDatexValueError(err.to_string()))?;
    serde_val.deserialize_into().map_err(|err| TryToDatexValueError(err.to_string()))
}

/// Converts a [Value] into a rust value by first converting it to a [serde_value::Value] and then deserializing it into the target type.
pub fn try_serde_from_value<'de, T: Deserialize<'de>>(
    value: Value,
) -> Result<T, TryFromDatexValueError> {
    let serde_val = serde_value::to_value(value).map_err(|err| TryFromDatexValueError(err.to_string()))?;
    T::deserialize(serde_val).map_err(|err| TryFromDatexValueError(err.to_string()))
}