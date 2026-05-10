pub mod serde_mapping;
pub mod shared;

use crate::values::value_container::ValueContainer;
use serde::{Serialize, de::DeserializeOwned};
use crate::compiler::error::SpannedCompilerError;
use crate::parser::errors::SpannedParserError;
use crate::runtime::execution::{ExecutionError};
use crate::runtime::{Runtime};
use crate::runtime::execution::context::ScriptExecutionError;

#[derive(Debug, Clone)]
pub struct TryFromValueContainerError(pub String);

#[derive(Debug, Clone)]
pub struct TryToValueContainerError(pub String);


#[derive(Debug)]
pub enum DeserializationError {
    NoValue,
    InvalidValue(TryFromValueContainerError),
    ExecutionError(ExecutionError),
    CanNotReadFile(String),
    #[cfg(feature = "parser")]
    ParserError(SpannedParserError),
    #[cfg(feature = "compiler")]
    CompilerError(SpannedCompilerError),
    NoStaticValueFound,
}

impl From<ExecutionError> for DeserializationError {
    fn from(err: ExecutionError) -> DeserializationError {
        DeserializationError::ExecutionError(err)
    }
}

#[cfg(feature = "parser")]
impl From<SpannedParserError> for DeserializationError {
    fn from(err: SpannedParserError) -> DeserializationError {
        DeserializationError::ParserError(err)
    }
}

impl From<TryFromValueContainerError> for DeserializationError {
    fn from(err: TryFromValueContainerError) -> DeserializationError {
        DeserializationError::InvalidValue(err)
    }
}

impl From<ScriptExecutionError> for DeserializationError {
    fn from(err: ScriptExecutionError) -> DeserializationError {
        match err {
            ScriptExecutionError::ExecutionError(e) => DeserializationError::ExecutionError(e),
            #[cfg(feature = "compiler")]
            ScriptExecutionError::CompilerError(e) => DeserializationError::CompilerError(e),
        }
    }
}


/// Base DATEX trait for value proxy. Must implement [DatexProxyDeserialize] and [DatexProxySerialize]
pub trait DatexProxy:
    Sized + DatexProxyDeserialize + DatexProxySerialize
{
}

/// Conversion from a [ValueContainer] to a rust value
pub trait DatexProxyDeserialize: Sized {
    fn try_from_value_container(value: ValueContainer) -> Result<Self, TryFromValueContainerError>;

    /// Deserialize a value of type T from a byte slice containing DXB data
    fn try_from_bytes(
        input: &[u8],
        runtime: &Runtime
    ) -> Result<Self, DeserializationError> {
        let value = runtime.execute_dxb_sync(&input, None, true)?;
        if let Some(value) = value {
            let config= Self::try_from_value_container(value)?;
            Ok(config)
        } else {
            Err(DeserializationError::NoValue)
        }
    }

    #[cfg(feature = "compiler")]
    fn try_from_script(
        script: &str,
        runtime: &Runtime
    ) -> Result<Self, DeserializationError> {
        let value = runtime.execute_sync(&script, &[], None)?;
        if let Some(value) = value {
            let config= Self::try_from_value_container(value)?;
            Ok(config)
        } else {
            Err(DeserializationError::NoValue)
        }
    }

    #[cfg(all(feature = "std", feature = "compiler"))]
    fn try_from_dx_file(
        path: &std::path::Path,
        runtime: &Runtime,
    ) -> Result<Self, DeserializationError> {
        let script = std::fs::read_to_string(path).map_err(|e| DeserializationError::CanNotReadFile(e.to_string()))?;
        Self::try_from_script(&script, runtime)
    }

    /// Create a value from a DX script string
    /// This will extract a static value from the script without executing it
    /// and use that value for deserialization
    /// If no static value is found, an error is returned
    /// This is useful for deserializing simple values like integer, text, map and list
    /// without the need to execute the script
    /// Note: This does not support expressions or computations in the script
    /// For example, the script `{ "key": 42 }` will work, but the script `{ "key": 40 + 2 }` will not
    /// because the latter requires execution to evaluate the expression
    /// and extract the value
    #[cfg(feature = "compiler")]
    fn try_from_static_script(script: &str) -> Result<Self, DeserializationError> {
        let value = crate::compiler::extract_static_value_from_script(script)?
            .ok_or(DeserializationError::NoStaticValueFound)?;
        Ok(Self::try_from_value_container(value)?)
    }
}

/// Conversion from a rust value to a [ValueContainer]. Might fail if serde values are serialized.
pub trait DatexProxySerialize {
    fn try_to_value_container(self) -> Result<ValueContainer, TryToValueContainerError>;
}

/// Infallible conversion from a rust value to a [ValueContainer].
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
    default fn try_to_value_container(self) -> Result<ValueContainer, TryToValueContainerError> {
        let serde_val = serde_value::to_value(self).map_err(|err| TryToValueContainerError(err.to_string()))?;
        serde_val.deserialize_into().map_err(|err| TryToValueContainerError(err.to_string()))
    }
}

impl<T> DatexProxyDeserialize for T
where
    T: DeserializeOwned,
{
    /// Converts a [ValueContainer] into a [DeserializeOwned] type by first converting it to a [serde_value::Value] and then deserializing it into the target type.
    default fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromValueContainerError> {
        let serde_val = serde_value::to_value(value).map_err(|err| TryFromValueContainerError(err.to_string()))?;
        T::deserialize(serde_val).map_err(|err| TryFromValueContainerError(err.to_string()))
    }
}
