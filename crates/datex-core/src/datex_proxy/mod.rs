pub mod serde_compat;
pub mod shared;

#[cfg(feature = "compiler")]
use crate::compiler::error::SpannedCompilerError;
#[cfg(feature = "parser")]
use crate::parser::errors::SpannedParserError;
use crate::{
    core_compiler::core_compilation_context::DXBWithSharedValues,
    prelude::*,
    runtime::{
        Runtime,
        cache::shared_references_cache::SharedReferencesCache,
        execution::{ExecutionError, context::ScriptExecutionError},
    },
    shared_values::errors::KeyNotFoundError,
    types::r#type::Type,
    values::{value::Value, value_container::ValueContainer},
};

#[cfg(feature = "decompiler")]
use crate::decompiler::{DecompileOptions, decompile_value};

#[derive(Debug, Clone)]
pub struct TryFromDatexValueError(pub String);

#[derive(Debug, Clone)]
pub struct TryToDatexValueError(pub String);

#[derive(Debug)]
pub enum DeserializationError {
    NoValue,
    InvalidValue(Box<TryFromDatexValueError>),
    ExecutionError(Box<ExecutionError>),
    CanNotReadFile(String),
    #[cfg(feature = "parser")]
    ParserError(Box<SpannedParserError>),
    #[cfg(feature = "compiler")]
    CompilerError(Box<SpannedCompilerError>),
    NoStaticValueFound,
}

impl From<ExecutionError> for DeserializationError {
    fn from(err: ExecutionError) -> DeserializationError {
        DeserializationError::ExecutionError(Box::new(err))
    }
}

#[cfg(feature = "parser")]
impl From<SpannedParserError> for DeserializationError {
    fn from(err: SpannedParserError) -> DeserializationError {
        DeserializationError::ParserError(Box::new(err))
    }
}

impl From<TryFromDatexValueError> for DeserializationError {
    fn from(err: TryFromDatexValueError) -> DeserializationError {
        DeserializationError::InvalidValue(Box::new(err))
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
pub trait DatexValueContainerProxy<C>:
    Sized
    + DatexValueContainerProxyDeserialize
    + DatexValueContainerProxySerialize<C>
    + DatexProxyTypes<C>
{
}

/// Base DATEX value Proxy trait - converts to and from [Value]
pub trait DatexValueProxy<C>:
    Sized
    + DatexValueProxyDeserialize
    + DatexValueProxySerialize<C>
    + DatexProxyTypes<C>
{
}

pub trait DatexValueContainerProxySerde<C>:
    Sized
    + DatexValueContainerProxySerialize<C>
    + DatexValueContainerProxyDeserialize
{
}

/// Trait for providing DATEX type information for a DatexProxy type
pub trait DatexProxyTypes<C> {
    fn datex_type(context: &mut C) -> Type;
}

/// Conversion from a [ValueContainer] to a rust value
pub trait DatexValueContainerProxyDeserialize: Sized {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError>;

    fn try_from_map_property(
        value: Result<ValueContainer, KeyNotFoundError>,
    ) -> Result<Self, TryFromDatexValueError> {
        let value =
            value.map_err(|err| TryFromDatexValueError(err.to_string()))?;

        Self::try_from_value_container(value)
    }

    /// Deserialize a value of type T from a byte slice containing DXB data
    fn try_from_bytes(
        dxb: Vec<u8>,
        runtime: &Runtime,
    ) -> Result<Self, DeserializationError> {
        let value = runtime.execute_dxb_sync(
            DXBWithSharedValues::new(dxb, vec![]),
            None,
            None,
            true,
        )?;
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

    fn try_from_map_property(
        value: Result<Value, KeyNotFoundError>,
    ) -> Result<Self, TryFromDatexValueError> {
        let value =
            value.map_err(|err| TryFromDatexValueError(err.to_string()))?;

        Self::try_from_value(value)
    }
}

/// Conversion from a rust value to a [ValueContainer]. Might fail if serde values are serialized.
pub trait DatexValueContainerProxySerialize<T> {
    fn try_to_value_container(
        self,
        context: &mut T,
    ) -> Result<ValueContainer, TryToDatexValueError>;

    #[cfg(feature = "decompiler")]
    fn try_to_datex_string(
        self,
        decompile_options: DecompileOptions,
        context: &mut T,
    ) -> Result<String, TryToDatexValueError>
    where
        Self: Sized,
    {
        Ok(decompile_value(
            &self.try_to_value_container(context)?,
            decompile_options,
        ))
    }
}

/// Conversion from a rust value to a [Value]. Might fail if serde values are serialized.
pub trait DatexValueProxySerialize<T> {
    fn try_to_value(
        self,
        context: &mut T,
    ) -> Result<Value, TryToDatexValueError>;
}

/// Infallible conversion from a rust value to a [ValueContainer].
/// Only works if no serde values are serialized.
pub trait DatexValueContainerProxyInfallibleSerialize<C> {
    fn to_value_container(self, context: &mut C) -> ValueContainer;

    #[cfg(feature = "decompiler")]
    fn to_datex_string(
        self,
        decompile_options: DecompileOptions,
        context: &mut C,
    ) -> String
    where
        Self: Sized,
    {
        decompile_value(&self.to_value_container(context), decompile_options)
    }
}

/// Infallible conversion from a rust value to a [Value].
/// Only works if no serde values are serialized.
pub trait DatexValueProxyInfallibleSerialize<C> {
    fn to_value(self, context: &mut C) -> Value;
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
    fn try_from_map_property(
        value: Result<ValueContainer, KeyNotFoundError>,
    ) -> Result<Self, TryFromDatexValueError> {
        match value {
            Ok(ValueContainer::Local(value)) => {
                DatexValueProxyDeserialize::try_from_map_property(Ok(value))
            }

            Ok(_) => Err(TryFromDatexValueError("Cannot cast from ValueContainer::Shared, expected ValueContainer::Local".to_string())),

            Err(err) => {
                DatexValueProxyDeserialize::try_from_map_property(Err(err))
            }
        }
    }
}

impl<T, C> DatexValueContainerProxySerialize<C> for T
where
    T: DatexValueProxySerialize<C>,
{
    default fn try_to_value_container(
        self,
        context: &mut C,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        DatexValueProxySerialize::try_to_value(self, context)
            .map(ValueContainer::from)
    }
}

impl<T, C> DatexValueContainerProxyInfallibleSerialize<C> for T
where
    T: DatexValueProxyInfallibleSerialize<C>,
{
    default fn to_value_container(self, context: &mut C) -> ValueContainer {
        ValueContainer::from(DatexValueProxyInfallibleSerialize::to_value(
            self, context,
        ))
    }
}

impl<T: DatexValueProxy<C>, C> DatexValueContainerProxy<C> for T {}

pub trait DatexValueContainerProxySerializeWithoutContext: Sized {
    fn try_to_value_container_without_context(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError>;
}

pub trait DatexValueProxySerializeWithoutContext: Sized {
    fn try_to_value_without_context(
        self,
    ) -> Result<Value, TryToDatexValueError>;
}

pub trait DatexTypeWithoutContext: Sized {
    fn datex_type_without_context() -> Type;
}

pub trait DatexValueContainerProxyInfallibleSerializeWithoutContext:
    Sized
{
    fn to_value_container_without_context(self) -> ValueContainer;
}
pub trait DatexValueProxyInfallibleSerializeWithoutContext: Sized {
    fn to_value_without_context(self) -> Value;
}

impl<T> DatexValueContainerProxySerializeWithoutContext for T
where
    T: DatexValueContainerProxySerialize<()>,
{
    fn try_to_value_container_without_context(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        self.try_to_value_container(&mut ())
    }
}

impl<T> DatexValueProxySerializeWithoutContext for T
where
    T: DatexValueProxySerialize<()>,
{
    fn try_to_value_without_context(
        self,
    ) -> Result<Value, TryToDatexValueError> {
        self.try_to_value(&mut ())
    }
}

impl<T> DatexValueContainerProxyInfallibleSerializeWithoutContext for T
where
    T: DatexValueContainerProxyInfallibleSerialize<()>,
{
    fn to_value_container_without_context(self) -> ValueContainer {
        self.to_value_container(&mut ())
    }
}
impl<T> DatexValueProxyInfallibleSerializeWithoutContext for T
where
    T: DatexValueProxyInfallibleSerialize<()>,
{
    fn to_value_without_context(self) -> Value {
        self.to_value(&mut ())
    }
}

impl<T> DatexTypeWithoutContext for T
where
    T: DatexProxyTypes<()>,
{
    fn datex_type_without_context() -> Type {
        Self::datex_type(&mut ())
    }
}
