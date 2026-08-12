use core::fmt::Debug;
use core::hash::Hash;
use crate::{
    core_compiler::core_compilation_context::DXBWithSharedValues,
    prelude::*,
    runtime::{
        Runtime,
        execution::{
            context::{ExecutionContext, ExecutionMode, LocalExecutionContext},
            execution_input::ExecutionCallerMetadata,
        },
    },
    types::type_definition::callable::CallableTypeDefinition,
    values::{
        core_values::{callable::error::CallableError, endpoint::Endpoint},
        value_container::ValueContainer,
    },
};

pub mod apply;
pub mod equality;
pub mod error;
mod serde_dif;

#[derive(Clone)]
pub struct NativeCallable {
    pub function: Rc<dyn Fn(Vec<ValueContainer>) -> Result<Option<ValueContainer>, CallableError> + 'static>
}

impl Debug for NativeCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativeCallable")
    }
}
impl PartialEq for NativeCallable {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.function, &other.function)
    }
}
impl Eq for NativeCallable {}

impl Hash for NativeCallable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(&self.function) as *const ();
        ptr.hash(state);
    }
}


impl NativeCallable {
    pub fn new(
        function: impl Fn(Vec<ValueContainer>) -> Result<Option<ValueContainer>, CallableError> + 'static,
    ) -> Self {
        NativeCallable {
            function: Rc::new(function),
        }
    }
}

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
        let stack_values = args
            .iter()
            .chain(self.injected_values.iter())
            .cloned()
            .collect::<Vec<_>>();

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

impl CallableBody {
    pub fn native(native_callable: impl Fn(Vec<ValueContainer>) -> Result<Option<ValueContainer>, CallableError> + 'static) -> Self {
        CallableBody::Native(NativeCallable::new(native_callable))
    }
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
            CallableBody::Native(native_callable) => (native_callable.function)(args),
            CallableBody::DatexBytecode(bytecode_callable) => {
                bytecode_callable.call(runtime, args)
            }
            CallableBody::CoreStub(_stub) => {
                Err(CallableError::RuntimeOnlyCallable)
            }
        }
    }
}
