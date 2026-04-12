use core::fmt::Display;
use crate::values::core_values::map::MapAccessError;
use crate::values::core_values::r#type::Type;
use crate::values::value_container::ValueContainer;

#[derive(Debug)]
pub struct IndexOutOfBoundsError {
    pub index: u32,
}

impl Display for IndexOutOfBoundsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Index out of bounds: {}", self.index)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyNotFoundError {
    pub key: ValueContainer,
}

impl Display for KeyNotFoundError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Property not found: {}", self.key)
    }
}

#[derive(Debug)]
pub enum AccessError {
    ImmutableReference,
    InvalidOperation(String),
    KeyNotFound(KeyNotFoundError),
    IndexOutOfBounds(IndexOutOfBoundsError),
    MapAccessError(MapAccessError),
    InvalidIndexKey,
}

impl From<IndexOutOfBoundsError> for AccessError {
    fn from(err: IndexOutOfBoundsError) -> Self {
        AccessError::IndexOutOfBounds(err)
    }
}

impl From<MapAccessError> for AccessError {
    fn from(err: MapAccessError) -> Self {
        AccessError::MapAccessError(err)
    }
}

impl From<KeyNotFoundError> for AccessError {
    fn from(err: KeyNotFoundError) -> Self {
        AccessError::KeyNotFound(err)
    }
}

impl Display for AccessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AccessError::MapAccessError(err) => {
                write!(f, "Map access error: {}", err)
            }
            AccessError::ImmutableReference => {
                write!(f, "Cannot modify an immutable reference")
            }
            AccessError::InvalidOperation(op) => {
                write!(f, "Invalid operation: {}", op)
            }
            AccessError::KeyNotFound(key) => {
                write!(f, "{}", key)
            }
            AccessError::IndexOutOfBounds(error) => {
                write!(f, "{}", error)
            }
            AccessError::InvalidIndexKey => {
                write!(f, "Invalid index key")
            }
        }
    }
}

#[derive(Debug)]
pub enum TypeError {
    TypeMismatch { expected: Type, found: Type },
}
impl Display for TypeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TypeError::TypeMismatch { expected, found } => write!(
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
    TypeError(Box<TypeError>),
}

impl Display for AssignmentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AssignmentError::ImmutableReference => {
                write!(f, "Cannot assign to an immutable reference")
            }
            AssignmentError::TypeError(e) => {
                write!(f, "Type error: {}", e)
            }
        }
    }
}