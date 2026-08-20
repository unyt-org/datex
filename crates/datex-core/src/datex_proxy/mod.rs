pub mod serde_compat;
pub mod shared;

#[cfg(feature = "compiler")]
use crate::compiler::error::SpannedCompilerError;
#[cfg(feature = "decompiler")]
use crate::decompiler::{DecompileOptions, decompile_value};
#[cfg(feature = "parser")]
use crate::parser::errors::SpannedParserError;
use crate::{
    core_compiler::core_compilation_context::DXBWithSharedValues,
    datex_proxy::shared::Shared,
    prelude::*,
    runtime::{
        Runtime,
        cache::shared_references_cache::SharedReferencesCache,
        execution::{ExecutionError, context::ScriptExecutionError},
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::errors::KeyNotFoundError,
    types::r#type::Type,
    values::{
        core_value::{CoreValue, DatexNative},
        value::Value,
        value_container::ValueContainer,
    },
};
use core::cell::{Ref, RefMut};

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
pub trait DatexValueContainerProxy:
    Sized
    + DatexValueContainerProxyDeserialize
    + DatexValueContainerProxySerialize
    + DatexProxyTypes
{
}

/// Base DATEX value Proxy trait - converts to and from [Value]
pub trait DatexValueProxy:
    Sized
    + DatexValueProxyDeserialize
    + DatexValueProxySerialize
    + DatexProxyTypes
{
}

/// Trait for providing DATEX type information for a DatexProxy type
/// FIXME: currently we need to implement both DatexProxyTypes<()> and DatexProxyTypes<SharedReferencesCache>
/// manually, generic default impl does not work
pub trait DatexProxyTypes {
    fn datex_type(context: &mut SharedReferencesCache) -> Type;
}

/// Conversion from a [ValueContainer] to a rust value
pub trait DatexValueContainerProxyDeserialize: Sized {
    /// Try to deserialize the given [ValueContainer] into Self.
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError>;

    /// Try to get a reference to Self from the given [Value].
    /// [CoreValue::Native] values can actually be borrowed, for other values, [None] is returned.
    fn try_borrow_mut_from_value_container(
        value: &mut ValueContainer,
    ) -> Option<&mut Self>
    where
        Self: Sized + 'static,
    {
        // try to downcast directly from native value
        if let ValueContainer::Local(Value {
            inner: CoreValue::Native(native),
            ..
        }) = value
            && let Some(native) = native.as_any_mut().downcast_mut::<Self>()
        {
            Some(native)
        } else {
            None
        }
    }

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
pub trait DatexValueProxyDeserialize {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError>
    where
        Self: Sized;

    /// Try to get a reference to Self from the given [Value].
    /// [CoreValue::Native] values can actually be borrowed, for other values, [None] is returned.
    fn try_borrow_mut_from_value(value: &mut Value) -> Option<&Self>
    where
        Self: Sized + 'static,
    {
        // try to downcast directly from native value
        if let CoreValue::Native(native) = &mut value.inner
            && let Some(native) = native.as_any_mut().downcast_mut::<Self>()
        {
            Some(native)
        } else {
            None
        }
    }

    fn try_from_map_property(
        value: Result<Value, KeyNotFoundError>,
    ) -> Result<Self, TryFromDatexValueError>
    where
        Self: Sized,
    {
        let value =
            value.map_err(|err| TryFromDatexValueError(err.to_string()))?;

        Self::try_from_value(value)
    }
}

/// Conversion from a rust value to a [ValueContainer]. Might fail if serde values are serialized.
pub trait DatexValueContainerProxySerialize {
    fn try_to_value_container(
        self,
        context: &mut SharedReferencesCache,
    ) -> Result<ValueContainer, TryToDatexValueError>;

    #[cfg(feature = "decompiler")]
    fn try_to_datex_string(
        self,
        decompile_options: DecompileOptions,
        context: &mut SharedReferencesCache,
    ) -> Result<String, TryToDatexValueError>
    where
        Self: Sized,
    {
        Ok(decompile_value(
            &Box::new(self).try_to_value_container(context)?,
            decompile_options,
        ))
    }
}

/// Conversion from a rust value to a [Value]. Might fail if serde values are serialized.
pub trait DatexValueProxySerialize {
    fn try_to_value(
        self,
        context: &mut SharedReferencesCache,
    ) -> Result<Value, TryToDatexValueError>;

    fn datex_instance_type(&self, context: &mut SharedReferencesCache) -> Type;

    fn try_shared(
        self,
        address_provider: &mut SelfOwnedPointerAddressProvider,
        context: &mut SharedReferencesCache,
    ) -> Result<Shared<Self>, TryToDatexValueError>
    where
        Self: DatexNative + Sized + DatexProxyTypes,
    {
        let ty = <Self as DatexProxyTypes>::datex_type(context);
        Shared::try_new(self, ty.convert_to_definition(), address_provider)
    }
}

/// Infallible conversion from a rust value to a [ValueContainer].
/// Only works if no serde values are serialized.
pub trait DatexValueContainerProxyInfallibleSerialize {
    fn to_value_container(self, context: &mut SharedReferencesCache) -> ValueContainer;

    #[cfg(feature = "decompiler")]
    fn to_datex_string(
        self,
        decompile_options: DecompileOptions,
        context: &mut SharedReferencesCache,
    ) -> String
    where
        Self: Sized,
    {
        decompile_value(&Box::new(self).to_value_container(context), decompile_options)
    }
}

/// Infallible conversion from a rust value to a [Value].
/// Only works if no serde values are serialized.
pub trait DatexValueProxyInfallibleSerialize {
    fn to_value(self, context: &mut SharedReferencesCache) -> Value;

    fn shared(
        self,
        address_provider: &mut SelfOwnedPointerAddressProvider,
        context: &mut SharedReferencesCache,
    ) -> Shared<Self>
    where
        Self: DatexNative + Sized + DatexProxyTypes,
    {
        let ty = <Self as DatexProxyTypes>::datex_type(context);
        Shared::new(self, ty.convert_to_definition(), address_provider)
    }
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

impl<T> DatexValueContainerProxySerialize for T
where
    T: DatexValueProxySerialize,
{
    default fn try_to_value_container(
        self,
        context: &mut SharedReferencesCache,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        DatexValueProxySerialize::try_to_value(self, context)
            .map(ValueContainer::from)
    }
}

impl<T> DatexValueContainerProxyInfallibleSerialize for T
where
    T: DatexValueProxyInfallibleSerialize,
{
    default fn to_value_container(self, context: &mut SharedReferencesCache) -> ValueContainer {
        ValueContainer::from(DatexValueProxyInfallibleSerialize::to_value(
            self, context,
        ))
    }
}

impl<T: DatexValueProxy> DatexValueContainerProxy for T {}