//! This module contains the implementation of the runtime, which is the backbone of the DATEX system.
use crate::{
    core_compiler::core_compilation_context::DXBWithSharedValues,
    runtime::{
        execution::{ExecutionError, context::RemoteExecutionTarget},
        remote_value_sync::SubscriberError,
    },
    values::value_container::ValueContainer,
};

use crate::prelude::*;
use core::{fmt::Debug, ops::Deref, result::Result};
use execution::context::{ExecutionContext, ScriptExecutionError};
pub mod cache;
mod config;
pub mod execution;
mod incoming_sections;
mod internal;
mod logger;
pub mod pointer_address_provider;
pub mod pointer_availability_lookup;
pub mod remote_value_sync;
mod runner;
#[cfg(test)]
pub mod test_utils;

use crate::{
    core_compiler::instruction_input::InstructionInput,
    values::core_values::endpoint::Endpoint,
};
pub use config::*;
pub use internal::*;
pub use runner::*;
use crate::runtime::execution::context::{ExecutionMode, FinalOrPersistedExecutionContext};
use crate::runtime::execution::execution_input::ExecutionCallerMetadata;

#[derive(Clone, Debug)]
pub struct Runtime {
    internal: Rc<RuntimeInternal>,
}

impl Deref for Runtime {
    type Target = RuntimeInternal;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}


/// publicly exposed wrapper impl for the Runtime
/// around RuntimeInternal
impl Runtime {
    pub(crate) fn new(runtime_internal: RuntimeInternal) -> Runtime {
        Runtime {
            internal: Rc::new(runtime_internal),
        }
    }

    fn register_endpoint_property(&self, name: &str, value: ValueContainer) {
        self.internal.register_endpoint_property(name, value);
    }

    pub fn stub() -> Runtime {
        Runtime::new(RuntimeInternal::stub())
    }

    fn internal(&self) -> Rc<RuntimeInternal> {
        Rc::clone(&self.internal)
    }

    #[cfg(feature = "compiler")]
    pub async fn execute<'a>(
        &'a self,
        script: &'a str,
        inserted_values: &'a [ValueContainer],
        execution_context: Option<impl Into<FinalOrPersistedExecutionContext<'a>>>,
    ) -> Result<Option<ValueContainer>, ScriptExecutionError> {
        RuntimeInternal::execute(
            self.internal(),
            script,
            inserted_values,
            execution_context,
        )
        .await
    }

    #[cfg(feature = "compiler")]
    pub fn execute_sync<'a>(
        &'a self,
        script: &'a str,
        inserted_values: &'a [ValueContainer],
        execution_context: Option<impl Into<FinalOrPersistedExecutionContext<'a>>>,
    ) -> Result<Option<ValueContainer>, ScriptExecutionError> {
        RuntimeInternal::execute_sync(
            self.internal(),
            script,
            inserted_values,
            execution_context,
        )
    }

    pub async fn execute_dxb<'a>(
        &'a self,
        input: DXBWithSharedValues,
        initial_stack_values: Option<Vec<ValueContainer>>,
        execution_context: Option<impl Into<FinalOrPersistedExecutionContext<'a>> + 'a>,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        RuntimeInternal::execute_dxb(
            self.internal(),
            input,
            initial_stack_values,
            execution_context,
        )
        .await
    }

    pub fn execute_dxb_sync<'a>(
        &'a self,
        input: DXBWithSharedValues,
        initial_stack_values: Option<Vec<ValueContainer>>,
        execution_context: Option<impl Into<FinalOrPersistedExecutionContext<'a>>>,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        RuntimeInternal::execute_dxb_sync(
            self.internal(),
            input,
            initial_stack_values,
            execution_context,
        )
    }

    pub async fn execute_remote(
        &self,
        remote_execution_target: &RemoteExecutionTarget,
        input: DXBWithSharedValues,
        is_end_of_context: bool,
    ) -> Result<Vec<(RemoteExecutionResult, Endpoint)>, SubscriberError> {
        RuntimeInternal::execute_remote(
            self.internal(),
            remote_execution_target,
            input,
            is_end_of_context
        )
        .await
    }

    pub async fn execute_remote_single_receiver(
        &self,
        remote_execution_target: &RemoteExecutionTarget,
        input: DXBWithSharedValues,
        is_end_of_context: bool,
    ) -> RemoteExecutionResult {
        RuntimeInternal::execute_remote_single_receiver(
            self.internal(),
            remote_execution_target,
            input,
            is_end_of_context
        )
        .await
    }

    pub async fn execute_instructions_remote(
        &self,
        endpoints: Vec<Endpoint>,
        instructions_input: Vec<InstructionInput>,
    ) -> Result<Vec<(RemoteExecutionResult, Endpoint)>, SubscriberError> {
        RuntimeInternal::execute_instructions_remote(
            self.internal(),
            endpoints,
            instructions_input,
        )
        .await
    }

    pub async fn execute_instructions_remote_single_receiver(
        &self,
        endpoint: Endpoint,
        instructions_input: Vec<InstructionInput>,
    ) -> RemoteExecutionResult {
        RuntimeInternal::execute_instructions_remote_single_receiver(
            self.internal(),
            endpoint,
            instructions_input,
        )
        .await
    }

    pub(crate) fn get_or_create_execution_context<'a>(
        self: &Runtime, 
        ctx: Option<impl Into<FinalOrPersistedExecutionContext<'a>>>
    ) -> FinalOrPersistedExecutionContext<'a> {
        match ctx {
            Some(context) => {
                let mut context = context.into();
                // set current runtime in execution context if local execution context
                if let &mut ExecutionContext::Local(ref mut local_context) = context.as_mut() {
                    local_context.set_runtime_internal(self.clone());
                }
                context
            },
            None => {
                FinalOrPersistedExecutionContext::Final(
                    ExecutionContext::local(ExecutionMode::Static, self.clone(), ExecutionCallerMetadata::local_default())
                )
            }
        }
    }
}
