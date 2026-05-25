pub mod serde_compat;
pub mod shared;

#[cfg(feature = "compiler")]
use crate::compiler::error::SpannedCompilerError;
#[cfg(feature = "parser")]
use crate::parser::errors::SpannedParserError;
use crate::{
    prelude::*,
    runtime::{
        Runtime,
        execution::{ExecutionError, context::ScriptExecutionError},
    },
    values::{value::Value, value_container::ValueContainer},
};
use crate::runtime::memory::Memory;
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;

#[derive(Debug, Clone)]
pub struct TryFromDatexValueError(pub String);

#[derive(Debug, Clone)]
pub struct TryToDatexValueError(pub String);

#[derive(Debug)]
pub enum DeserializationError {
    NoValue,
    InvalidValue(TryFromDatexValueError),
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

impl From<TryFromDatexValueError> for DeserializationError {
    fn from(err: TryFromDatexValueError) -> DeserializationError {
        DeserializationError::InvalidValue(err)
    }
}

impl From<ScriptExecutionError> for DeserializationError {
    fn from(err: ScriptExecutionError) -> DeserializationError {
        match err {
            ScriptExecutionError::ExecutionError(e) => {
                DeserializationError::ExecutionError(e)
            }
            #[cfg(feature = "compiler")]
            ScriptExecutionError::CompilerError(e) => {
                DeserializationError::CompilerError(e)
            }
        }
    }
}

/// Base DATEX value Proxy trait - converts to and from [ValueContainer]
/// Must implement [DatexValueContainerProxyDeserialize] and [DatexValueContainerProxySerialize]
pub trait DatexValueContainerProxy:
    Sized + DatexValueContainerProxyDeserialize + DatexValueContainerProxySerialize + DatexProxyTypes
{
}

/// Base DATEX value Proxy trait - converts to and from [Value]
pub trait DatexValueProxy:
    Sized + DatexValueProxyDeserialize + DatexValueProxySerialize + DatexProxyTypes
{
}

/// Trait for providing DATEX type information for a DatexProxy type
pub trait DatexProxyTypes {
    fn datex_type(memory: &mut Memory) -> Type;
}


/// Conversion from a [ValueContainer] to a rust value
pub trait DatexValueContainerProxyDeserialize: Sized {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError>;

    /// Deserialize a value of type T from a byte slice containing DXB data
    fn try_from_bytes(
        input: &[u8],
        runtime: &Runtime,
    ) -> Result<Self, DeserializationError> {
        let value = runtime.execute_dxb_sync(input, None, true)?;
        if let Some(value) = value {
            let config = Self::try_from_value_container(value)?;
            Ok(config)
        } else {
            Err(DeserializationError::NoValue)
        }
    }

    #[cfg(feature = "compiler")]
    fn try_from_script(
        script: &str,
        runtime: &Runtime,
    ) -> Result<Self, DeserializationError> {
        let value = runtime.execute_sync(script, &[], None)?;
        if let Some(value) = value {
            let config = Self::try_from_value_container(value)?;
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
        let script = std::fs::read_to_string(path)
            .map_err(|e| DeserializationError::CanNotReadFile(e.to_string()))?;
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
    fn try_from_static_script(
        script: &str,
    ) -> Result<Self, DeserializationError> {
        let value = crate::compiler::extract_static_value_from_script(script)?
            .ok_or(DeserializationError::NoStaticValueFound)?;
        Ok(Self::try_from_value_container(value)?)
    }
}

/// Conversion from a [Value] to a rust value
pub trait DatexValueProxyDeserialize: Sized {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError>;
}

/// Conversion from a rust value to a [ValueContainer]. Might fail if serde values are serialized.
pub trait DatexValueContainerProxySerialize {
    fn try_to_value_container(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError>;
}

/// Conversion from a rust value to a [Value]. Might fail if serde values are serialized.
pub trait DatexValueProxySerialize {
    fn try_to_value(self) -> Result<Value, TryToDatexValueError>;
}

/// Infallible conversion from a rust value to a [ValueContainer].
/// Only works if no serde values are serialized.
pub trait DatexValueContainerProxyInfallibleSerialize {
    fn to_value_container(self) -> ValueContainer;
}

/// Infallible conversion from a rust value to a [Value].
/// Only works if no serde values are serialized.
pub trait DatexValueProxyInfallibleSerialize {
    fn to_value(self) -> Value;
}

// Blanket DatexValueContainerProxy trait impls for types that implement DatexValueProxy traits:
impl<T: DatexValueProxyDeserialize> DatexValueContainerProxyDeserialize for T {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        match value {
            ValueContainer::Local(val) => DatexValueProxyDeserialize::try_from_value(val),
            _ => Err(TryFromDatexValueError("Cannot cast from ValueContainer::Shared, expected ValueContainer::Local".to_string())),
        }
    }
}

impl<T: DatexValueProxySerialize> DatexValueContainerProxySerialize for T {
    fn try_to_value_container(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        DatexValueProxySerialize::try_to_value(self).map(ValueContainer::from)
    }
}

impl<T: DatexValueProxyInfallibleSerialize>
    DatexValueContainerProxyInfallibleSerialize for T
{
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::from(DatexValueProxyInfallibleSerialize::to_value(self))
    }
}

impl<T: DatexValueProxy> DatexValueContainerProxy for T {}
