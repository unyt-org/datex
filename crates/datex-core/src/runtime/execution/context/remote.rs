#[cfg(feature = "compiler")]
use crate::compiler::scope::CompilationScope;
use crate::{
    global::dxb_block::OutgoingContextId,
    runtime::{
        Runtime,
        execution::context::{ExecutionContext, ExecutionMode},
    },
    values::core_values::endpoint::Endpoint,
};

#[derive(Debug, Clone)]
pub struct RemoteExecutionContext {
    #[cfg(feature = "compiler")]
    pub compile_scope: CompilationScope,
    pub endpoint: Endpoint,
    pub context_id: Option<OutgoingContextId>,
    pub execution_mode: ExecutionMode,
    pub runtime: Runtime,
}

impl RemoteExecutionContext {
    /// Creates a new remote execution context with the given endpoint.
    pub fn new(
        endpoint: impl Into<Endpoint>,
        execution_mode: ExecutionMode,
        runtime: Runtime,
    ) -> Self {
        RemoteExecutionContext {
            #[cfg(feature = "compiler")]
            compile_scope: CompilationScope::new(execution_mode),
            endpoint: endpoint.into(),
            context_id: None,
            execution_mode,
            runtime,
        }
    }
}

impl ExecutionContext {
    pub fn remote(endpoint: impl Into<Endpoint>, runtime: Runtime) -> Self {
        ExecutionContext::Remote(RemoteExecutionContext::new(
            endpoint,
            ExecutionMode::Static,
            runtime,
        ))
    }

    pub fn remote_unbounded(
        endpoint: impl Into<Endpoint>,
        runtime: Runtime,
    ) -> Self {
        ExecutionContext::Remote(RemoteExecutionContext::new(
            endpoint,
            ExecutionMode::unbounded(),
            runtime,
        ))
    }
}
