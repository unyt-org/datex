use core::fmt::Display;
use crate::values::core_values::map::MapAccessError;
use crate::values::core_values::r#type::Type;
use crate::values::value_container::ValueContainer;

#[derive(Debug)]
pub struct IndexOutOfBoundsError {
    pub index: u32,
}

impl Display for crate::shared_values::shared_container::IndexOutOfBoundsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Index out of bounds: {}", self.index)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyNotFoundError {
    pub key: ValueContainer,
}

impl Display for crate::shared_values::shared_container::KeyNotFoundError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Property not found: {}", self.key)
    }
}

#[derive(Debug)]
pub enum AccessError {
    ImmutableReference,
    InvalidOperation(String),
    KeyNotFound(crate::shared_values::shared_container::KeyNotFoundError),
    IndexOutOfBounds(crate::shared_values::shared_container::IndexOutOfBoundsError),
    MapAccessError(MapAccessError),
    InvalidIndexKey,
}

impl From<crate::shared_values::shared_container::IndexOutOfBoundsError> for crate::shared_values::shared_container::AccessError {
    fn from(err: crate::shared_values::shared_container::IndexOutOfBoundsError) -> Self {
        crate::shared_values::shared_container::AccessError::IndexOutOfBounds(err)
    }
}

impl From<MapAccessError> for crate::shared_values::shared_container::AccessError {
    fn from(err: MapAccessError) -> Self {
        crate::shared_values::shared_container::AccessError::MapAccessError(err)
    }
}

impl From<crate::shared_values::shared_container::KeyNotFoundError> for crate::shared_values::shared_container::AccessError {
    fn from(err: crate::shared_values::shared_container::KeyNotFoundError) -> Self {
        crate::shared_values::shared_container::AccessError::KeyNotFound(err)
    }
}

impl Display for crate::shared_values::shared_container::AccessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            crate::shared_values::shared_container::AccessError::MapAccessError(err) => {
                write!(f, "Map access error: {}", err)
            }
            crate::shared_values::shared_container::AccessError::ImmutableReference => {
                write!(f, "Cannot modify an immutable reference")
            }
            crate::shared_values::shared_container::AccessError::InvalidOperation(op) => {
                write!(f, "Invalid operation: {}", op)
            }
            crate::shared_values::shared_container::AccessError::KeyNotFound(key) => {
                write!(f, "{}", key)
            }
            crate::shared_values::shared_container::AccessError::IndexOutOfBounds(error) => {
                write!(f, "{}", error)
            }
            crate::shared_values::shared_container::AccessError::InvalidIndexKey => {
                write!(f, "Invalid index key")
            }
        }
    }
}

#[derive(Debug)]
pub enum TypeError {
    TypeMismatch { expected: Type, found: Type },
}
impl Display for crate::shared_values::shared_container::TypeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            crate::shared_values::shared_container::TypeError::TypeMismatch { expected, found } => write!(
                f,
                "Type mismatch: expected {}, found {}",
                expected, found
            ),
        }
    }
}

#[derive(Debug)]
pub enum AssignmentError {
    ImmutableReference,
    TypeError(Box<crate::shared_values::shared_container::TypeError>),
}

impl Display for crate::shared_values::shared_container::AssignmentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            crate::shared_values::shared_container::AssignmentError::ImmutableReference => {
                write!(f, "Cannot assign to an immutable reference")
            }
            crate::shared_values::shared_container::AssignmentError::TypeError(e) => {
                write!(f, "Type error: {}", e)
            }
        }
    }
}