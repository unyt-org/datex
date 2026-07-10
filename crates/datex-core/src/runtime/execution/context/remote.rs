#[cfg(feature = "compiler")]
use crate::compiler::scope::CompilationScope;
use crate::{
    global::dxb_block::OutgoingContextId,
    prelude::*,
    runtime::{
        Runtime,
        execution::context::{ExecutionContext, ExecutionMode},
    },
    values::core_values::endpoint::Endpoint,
};
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct RemoteExecutionContext {
    #[cfg(feature = "compiler")]
    pub compile_scope: CompilationScope,
    pub endpoints: Vec<Endpoint>,
    pub context_id: RefCell<Option<OutgoingContextId>>,
    pub execution_mode: ExecutionMode,
    pub runtime: Runtime,
}

impl RemoteExecutionContext {
    /// Creates a new remote execution context with the given endpoint.
    pub fn new(
        endpoints: Vec<Endpoint>,
        execution_mode: ExecutionMode,
        runtime: Runtime,
    ) -> Self {
        RemoteExecutionContext {
            #[cfg(feature = "compiler")]
            compile_scope: CompilationScope::new(execution_mode),
            endpoints,
            context_id: RefCell::new(None),
            execution_mode,
            runtime,
        }
    }

    /// Adds an endpoint to the remote execution context if it doesn't already exist.
    pub fn add_endpoint(&mut self, endpoint: Endpoint) {
        if self.endpoints.contains(&endpoint) {
            return;
        }
        self.endpoints.push(endpoint);
    }
}

impl ExecutionContext {
    pub fn remote(endpoints: Vec<Endpoint>, runtime: Runtime) -> Self {
        ExecutionContext::Remote(RemoteExecutionContext::new(
            endpoints,
            ExecutionMode::Static,
            runtime,
        ))
    }

    pub fn remote_unbounded(
        endpoints: Vec<Endpoint>,
        runtime: Runtime,
    ) -> Self {
        ExecutionContext::Remote(RemoteExecutionContext::new(
            endpoints,
            ExecutionMode::unbounded(),
            runtime,
        ))
    }
}
