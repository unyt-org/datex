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
    pub target: RemoteExecutionTarget,
    pub execution_mode: ExecutionMode,
    pub runtime: Runtime,
}

#[derive(Debug, Clone)]
pub struct RemoteExecutionTarget {
    endpoints: Vec<Endpoint>,
    pub context_id: Rc<RefCell<Option<OutgoingContextId>>>,
}

impl RemoteExecutionTarget {
    pub fn endpoints(&self) -> &[Endpoint] {
        &self.endpoints
    }
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
            target: RemoteExecutionTarget {
                endpoints,
                context_id: Rc::new(RefCell::new(None)),
            },
            execution_mode,
            runtime,
        }
    }

    /// Adds an endpoint to the remote execution context if it doesn't already exist.
    pub fn add_endpoint(&mut self, endpoint: Endpoint) {
        if self.endpoints().contains(&endpoint) {
            return;
        }
        self.endpoints_mut().push(endpoint);
    }

    pub fn endpoints(&self) -> &[Endpoint] {
        &self.target.endpoints
    }
    pub fn endpoints_mut(&mut self) -> &mut Vec<Endpoint> {
        &mut self.target.endpoints
    }

    /// Sets the endpoints for the remote execution context.
    /// # Safety
    /// The caller must ensure that the endpoints vector is not empty.
    pub unsafe fn set_endpoints(&mut self, endpoints: Vec<Endpoint>) {
        if endpoints.is_empty() {
            panic!("RemoteExecutionContext: endpoints cannot be empty");
        } else {
            self.target.endpoints = endpoints;
        }
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
