use crate::types::r#type::Type;
use binrw::{BinRead, BinWrite};
use core::fmt::{Display, Formatter};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
pub mod serde_dif;
use crate::prelude::*;

#[derive(
    Clone,
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
    pub parameter_types: Vec<(Option<String>, Type)>,
    pub rest_parameter_type: Option<(Option<String>, Box<Type>)>,
    pub return_type: Option<Box<Type>>,
    pub yeet_type: Option<Box<Type>>,
}

impl Display for CallableTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let params = self
            .parameter_types
            .iter()
            .map(|(name, ty)| match name {
                Some(name) => format!("{}: {}", name, ty),
                None => format!("{}", ty),
            })
            .chain(self.rest_parameter_type.iter().map(
                |(name, ty)| match name {
                    Some(name) => format!("...{}: {}", name, ty),
                    None => format!("...{}", ty),
                },
            ))
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
