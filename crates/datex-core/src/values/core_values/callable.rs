use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    traits::{apply::Apply, structural_eq::StructuralEq},
    values::{core_values::r#type::Type, value_container::ValueContainer},
};
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    fn(&[ValueContainer]) -> Result<Option<ValueContainer>, ExecutionError>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CallableBody {
    Native(NativeCallable),
    DatexBytecode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallableSignature {
    pub kind: CallableKind,
    pub parameter_types: Vec<(Option<String>, Type)>,
    pub rest_parameter_type: Option<(Option<String>, Box<Type>)>,
    pub return_type: Option<Box<Type>>,
    pub yeet_type: Option<Box<Type>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Callable {
    pub name: Option<String>,
    pub signature: CallableSignature,
    pub body: CallableBody,
    pub bound_this: Option<Box<ValueContainer>>,
}

impl Callable {
    pub fn call(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let actual_args = if let Some(this) = &self.bound_this {
            let mut new_args = alloc::vec::Vec::with_capacity(args.len() + 1);
            new_args.push(*this.clone());
            new_args.extend_from_slice(args);
            new_args
        } else {
            args.to_vec()
        };

        match &self.body {
            CallableBody::Native(func) => func(&actual_args),
            CallableBody::DatexBytecode => {
                todo!("#606 Calling Datex bytecode is not yet implemented")
            }
        }
    }
}

impl Apply for Callable {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.call(args)
    }
    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.call(&[arg.clone()])
    }
}

impl StructuralEq for Callable {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
