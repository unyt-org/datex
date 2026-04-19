use core::fmt::Display;
use crate::global::operators::binary::ArithmeticOperator;
use crate::prelude::*;
use crate::types::r#type::Type;

#[derive(Debug)]
pub enum IllegalTypeError {
    MutableRef(String),
    TypeNotFound,
}

impl Display for IllegalTypeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IllegalTypeError::MutableRef(val) => {
                core::write!(f, "Cannot use mutable reference as type: {}", val)
            }
            IllegalTypeError::TypeNotFound => {
                core::write!(f, "Core type not found in memory")
            }
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeError {
    SubvariantNotFound(String, String),
    // only for debugging purposes
    InvalidUnboxType(Type),
    Unimplemented(String),
    MismatchedOperands(ArithmeticOperator, Type, Type),
    AssignmentToImmutableReference(String),
    AssignmentToImmutableValue(String),
    AssignmentToConstant(String),
    ReferenceToNonTypeValue,

    // can not assign value to variable of different type
    AssignmentTypeMismatch {
        expected: Type,
        found: Type,
    },
}

impl Display for TypeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TypeError::AssignmentToImmutableValue(var_name) => {
                write!(f, "Cannot assign to immutable variable '{}'", var_name)
            }
            TypeError::AssignmentToConstant(var_name) => {
                write!(f, "Cannot assign to constant variable '{}'", var_name)
            }
            TypeError::AssignmentToImmutableReference(var_name) => {
                write!(
                    f,
                    "Cannot assign to immutable reference variable '{}'",
                    var_name
                )
            }
            TypeError::SubvariantNotFound(ty, variant) => {
                write!(
                    f,
                    "Type {} does not have a subvariant named {}",
                    ty, variant
                )
            }
            TypeError::InvalidUnboxType(ty) => {
                write!(f, "Cannot unbox value of type {}", ty)
            }
            TypeError::Unimplemented(msg) => {
                write!(f, "Unimplemented type inference case: {}", msg)
            }
            TypeError::MismatchedOperands(op, lhs, rhs) => {
                write!(
                    f,
                    "Cannot perform \"{}\" operation on {} and {}",
                    op, lhs, rhs
                )
            }
            TypeError::AssignmentTypeMismatch {
                expected: annotated_type,
                found: assigned_type,
            } => {
                write!(
                    f,
                    "Cannot assign {} to {}",
                    assigned_type, annotated_type
                )
            }
            TypeError::ReferenceToNonTypeValue => {
                write!(
                    f,
                    "Invalid reference to non-type value"
                )
            }
        }
    }
}
