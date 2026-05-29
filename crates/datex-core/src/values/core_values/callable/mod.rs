use crate::{
    prelude::*,
    types::type_definition::callable::CallableTypeDefinition,
    values::{
        core_values::callable::error::CallableError,
        value_container::ValueContainer,
    },
};
pub mod apply;
pub mod equality;
pub mod error;

pub type NativeCallable =
    fn(&[ValueContainer]) -> Result<Option<ValueContainer>, CallableError>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CallableBody {
    Native(NativeCallable),
    DatexBytecode(Vec<u8>),
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
            CallableBody::DatexBytecode(_bytecode) => {
                todo!("#606 Calling Datex bytecode is not yet implemented")
            }
        }
    }
}
