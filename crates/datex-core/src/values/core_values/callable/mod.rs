use serde::{Deserialize, Serialize};

use crate::{
    prelude::*,
    types::{r#type::Type, type_definition::callable::CallableTypeDefinition},
    values::{
        core_values::callable::error::CallableError,
        value_container::ValueContainer,
    },
};
use core::fmt::{Display, Formatter};
pub mod apply;
pub mod equality;
pub mod error;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

pub type NativeCallable =
    fn(&[ValueContainer]) -> Result<Option<ValueContainer>, CallableError>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CallableBody {
    Native(NativeCallable),
    DatexBytecode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Callable {
    pub name: Option<String>,
    pub signature: CallableTypeDefinition,
    pub body: CallableBody,
}

impl Callable {
    pub fn call(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, CallableError> {
        match &self.body {
            CallableBody::Native(func) => func(args),
            CallableBody::DatexBytecode => {
                todo!("#606 Calling Datex bytecode is not yet implemented")
            }
        }
    }
}
