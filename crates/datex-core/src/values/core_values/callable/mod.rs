use crate::{
    prelude::*,
    runtime::Runtime,
    types::type_definition::callable::CallableTypeDefinition,
    values::{
        core_values::{callable::error::CallableError, endpoint::Endpoint},
        value_container::ValueContainer,
    },
};
use crate::core_compiler::core_compilation_context::DXBWithSharedValues;
use crate::runtime::execution::context::{ExecutionContext, ExecutionMode, LocalExecutionContext};
use crate::runtime::execution::execution_input::ExecutionCallerMetadata;

pub mod apply;
pub mod equality;
pub mod error;
mod serde_dif;

pub type NativeCallable =
    fn(Vec<ValueContainer>) -> Result<Option<ValueContainer>, CallableError>;


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DatexBytecodeCallable {
    pub injected_values: Vec<ValueContainer>,
    pub body: Vec<u8>,
}

impl DatexBytecodeCallable {
    pub fn call(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, CallableError> {
        // construct the initial stack values by combining the provided arguments with the injected values
        let stack_values = args.iter().chain(self.injected_values.iter()).cloned().collect::<Vec<_>>();

        Ok(runtime.execute_dxb_sync(
            DXBWithSharedValues::new(self.body.clone(), vec![]), // TODO: no clone?
            Some(stack_values),
            Some(&mut ExecutionContext::Local(LocalExecutionContext::new(
                ExecutionMode::Static,
                runtime.clone(),
                ExecutionCallerMetadata::local_default(), // TODO caller
            ))),
            true,
        )?)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CallableBody {
    Native(NativeCallable),
    DatexBytecode(DatexBytecodeCallable),
    CoreStub(CoreStub),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CoreStub {
    Panic,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Callable {
    pub name: Option<String>,
    pub signature: CallableTypeDefinition,
    pub body: CallableBody,
    pub creator: Endpoint,
}

impl Callable {
    pub fn call(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, CallableError> {
        match &self.body {
            CallableBody::Native(native_callable) => native_callable(args),
            CallableBody::DatexBytecode(bytecode_callable) => bytecode_callable.call(runtime, args),
            CallableBody::CoreStub(_stub) => {
                Err(CallableError::RuntimeOnlyCallable)
            }
        }
    }
}
