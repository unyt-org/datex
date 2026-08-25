use crate::types::r#type::Type;
use binrw::{BinRead, BinWrite};
use core::fmt::{Display, Formatter};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
pub mod serde_dif;
use crate::prelude::*;
use crate::traits::apply::ApplyArgument;
use crate::types::traits::type_match::TypeSatisfiesValueContainer;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    TryFromPrimitive,
    BinRead,
    BinWrite,
)]
#[brw(repr(u8))]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
pub enum CallableKind {
    // A pure function
    Function,
    // A procedure that may have side effects
    Procedure,
}

impl Display for CallableKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            CallableKind::Function => write!(f, "function"),
            CallableKind::Procedure => write!(f, "procedure"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallableTypeDefinition {
    pub kind: CallableKind,
    pub requires_async: bool,
    pub parameters: Vec<(Option<String>, Type)>,
    pub rest_parameter: Option<(Option<String>, Box<Type>)>,
    pub return_type: Option<Box<Type>>,
    pub yeet_type: Option<Box<Type>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InvalidArgumentError {
    InvalidType { arg_name: String, expected: Type, provided: Type },
    InvalidArgumentCount { expected: usize, provided: usize },
}

impl Display for InvalidArgumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            InvalidArgumentError::InvalidType { arg_name, expected, provided } => {
                write!(f, "Invalid type for {}: expected {}, got {}", arg_name, expected, provided)
            }
            InvalidArgumentError::InvalidArgumentCount { expected, provided } => {
                write!(f, "Invalid argument count: expected {}, got {}", expected, provided)
            }
        }
    }
}

impl CallableTypeDefinition {
    /// Validates the provided arguments against the callable's parameter definitions.
    pub fn validate_arguments(
        &self,
        args: &[ApplyArgument]
    ) -> Result<(), InvalidArgumentError> {
        let expected_count = self.parameters.len();
        
        // too many or too few arguments provided
        if args.len() != expected_count && !(
            // okay if there's a rest parameter and we have at least the required number of arguments
            self.rest_parameter.is_some() && args.len() >= expected_count 
        ) {
            return Err(InvalidArgumentError::InvalidArgumentCount {
                expected: expected_count,
                provided: args.len(),
            });
        }

        /// Check if the provided arguments match the expected types for each parameter.
        for (i, (param_name, param_type)) in self.parameters.iter().enumerate() {
            let arg = &args[i];
            if !param_type.satisfies_value_container(&arg.value) {
                return Err(InvalidArgumentError::InvalidType {
                    arg_name: param_name.clone().unwrap_or_else(|| format!("arg_{}", i)),
                    expected: param_type.clone(),
                    provided: Type::Definition(arg.value.actual_container_type()),
                });
            }
        }

        /// If there's a rest parameter, check the types of the remaining arguments.
        if let Some((rest_param_name, rest_param_type)) = &self.rest_parameter {
            for (i, arg) in args[self.parameters.len()..].iter().enumerate() {
                if !rest_param_type.satisfies_value_container(&arg.value) {
                    return Err(InvalidArgumentError::InvalidType {
                        arg_name: rest_param_name.clone().unwrap_or_else(|| format!("rest_arg_{}", i)),
                        expected: *rest_param_type.clone(),
                        provided: Type::Definition(arg.value.actual_container_type()),
                    });
                }
            }
        }

        Ok(())
    }
}


impl Display for CallableTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let params = self
            .parameters
            .iter()
            .map(|(name, ty)| match name {
                Some(name) => format!("{}: {}", name, ty),
                None => format!("{}", ty),
            })
            .chain(self.rest_parameter.iter().map(|(name, ty)| match name {
                Some(name) => format!("...{}: {}", name, ty),
                None => format!("...{}", ty),
            }))
            .collect::<Vec<_>>()
            .join(", ");
        let return_type = self
            .return_type
            .as_ref()
            .map(|ty| format!(" -> {}", ty))
            .unwrap_or_default();
        let yeet_type = self
            .yeet_type
            .as_ref()
            .map(|ty| format!(" yeets {}", ty))
            .unwrap_or_default();
        write!(f, "{}({}){}{}", self.kind, params, return_type, yeet_type)
    }
}
