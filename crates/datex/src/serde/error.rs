#[cfg(feature = "compiler")]
use crate::compiler::error::{CompilerError, SpannedCompilerError};
use crate::runtime::execution::ExecutionError;

use crate::prelude::*;
use core::{fmt, fmt::Display, prelude::rust_2024::*};
use serde::{
    de::Error,
    ser::{
        StdError, {self},
    },
};

#[derive(Debug)]
pub enum SerializationError {
    Custom(String),
    CanNotSerialize(String),
    #[cfg(feature = "compiler")]
    CompilerError(CompilerError),
}
impl ser::Error for SerializationError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        SerializationError::Custom(msg.to_string())
    }
}
impl Error for SerializationError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        SerializationError::Custom(msg.to_string())
    }
}

impl From<String> for SerializationError {
    fn from(e: String) -> Self {
        SerializationError::Custom(e)
    }
}

#[cfg(feature = "compiler")]
impl From<CompilerError> for SerializationError {
    fn from(e: CompilerError) -> Self {
        SerializationError::CompilerError(e)
    }
}
impl StdError for SerializationError {}
impl Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializationError::Custom(msg) => {
                core::write!(f, "Serialization error: {}", msg)
            }
            SerializationError::CanNotSerialize(msg) => {
                core::write!(f, "Can not serialize value: {}", msg)
            }
            #[cfg(feature = "compiler")]
            SerializationError::CompilerError(err) => {
                core::write!(f, "Compiler error: {}", err)
            }
        }
    }
}

#[derive(Debug)]
pub enum DeserializationError {
    Custom(String),
    CanNotDeserialize(String),
    ExecutionError(ExecutionError),
    CanNotReadFile(String),
    #[cfg(feature = "compiler")]
    CompilerError(SpannedCompilerError),
    NoStaticValueFound,
}
impl ser::Error for DeserializationError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeserializationError::Custom(msg.to_string())
    }
}
impl Error for DeserializationError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeserializationError::Custom(msg.to_string())
    }
}

impl From<String> for DeserializationError {
    fn from(e: String) -> Self {
        DeserializationError::Custom(e.to_string())
    }
}
impl From<ExecutionError> for DeserializationError {
    fn from(e: ExecutionError) -> Self {
        DeserializationError::ExecutionError(e)
    }
}

impl StdError for DeserializationError {}
impl Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeserializationError::Custom(msg) => {
                core::write!(f, "Deserialization error: {}", msg)
            }
            DeserializationError::CanNotDeserialize(msg) => {
                core::write!(f, "Can not deserialize value: {}", msg)
            }
            DeserializationError::ExecutionError(err) => {
                core::write!(f, "Execution error: {}", err)
            }
            DeserializationError::CanNotReadFile(msg) => {
                core::write!(f, "Can not read file: {}", msg)
            }
            #[cfg(feature = "compiler")]
            DeserializationError::CompilerError(err) => {
                core::write!(f, "Compiler error: {}", err)
            }
            DeserializationError::NoStaticValueFound => {
                core::write!(f, "No static value found in script")
            }
        }
    }
}
