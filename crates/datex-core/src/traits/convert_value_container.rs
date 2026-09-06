#[cfg(feature = "compiler")]
use crate::compiler::error::SpannedCompilerError;
#[cfg(feature = "parser")]
use crate::parser::errors::SpannedParserError;
use crate::{
    core_compiler::core_compilation_context::DXBWithSharedValues,
    prelude::*,
    preludes::derive::{SharedReferencesCache, ValueContainer},
    runtime::{
        Runtime,
        execution::{ExecutionError, context::ScriptExecutionError},
    },
    values::borrowed_value_container::BorrowedValueContainer,
};

#[derive(Debug)]
pub enum DeserializationError {
    NoValue,
    InvalidValue,
    ExecutionError(Box<ExecutionError>),
    CanNotReadFile(String),
    #[cfg(feature = "parser")]
    ParserError(SpannedParserError),
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
        DeserializationError::ParserError(err)
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

pub trait ConvertValueContainer {
    fn to_value_container(
        self,
        cache: &mut SharedReferencesCache,
    ) -> ValueContainer;

    fn as_borrowed_value_container(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainer<'_>;

    fn try_from_value_container(
        value_container: ValueContainer,
    ) -> Result<Self, ValueContainer>
    where
        Self: Sized;

    fn try_borrow_from_value_container(
        value_container: &ValueContainer,
    ) -> Result<&Self, ()>
    where
        Self: Sized;

    fn try_borrow_mut_from_value_container(
        value_container: &mut ValueContainer,
    ) -> Result<&mut Self, ()>
    where
        Self: Sized;

    /// Deserialize a value of type T from a byte slice containing DXB data
    fn try_from_bytes(
        dxb: Vec<u8>,
        runtime: &Runtime,
    ) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let value = runtime.execute_dxb_sync(
            DXBWithSharedValues::new(dxb, vec![]),
            None,
            None,
            true,
        )?;
        if let Some(value) = value {
            let config = Self::try_from_value_container(value)
                .map_err(|_| DeserializationError::InvalidValue)?;
            Ok(config)
        } else {
            Err(DeserializationError::NoValue)
        }
    }

    #[cfg(feature = "compiler")]
    fn try_from_script(
        script: &str,
        runtime: &Runtime,
    ) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let value = runtime.execute_sync(script, &[], None)?;
        if let Some(value) = value {
            let config = Self::try_from_value_container(value)
                .map_err(|_| DeserializationError::InvalidValue)?;
            Ok(config)
        } else {
            Err(DeserializationError::NoValue)
        }
    }

    #[cfg(all(feature = "std", feature = "compiler"))]
    fn try_from_dx_file(
        path: &std::path::Path,
        runtime: &Runtime,
    ) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
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
    ) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let value = crate::compiler::extract_static_value_from_script(script)?
            .ok_or(DeserializationError::NoStaticValueFound)?;
        Self::try_from_value_container(value)
            .map_err(|_| DeserializationError::InvalidValue)
    }
}
