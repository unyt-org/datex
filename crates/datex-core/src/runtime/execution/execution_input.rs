use crate::{
    core_compiler::core_compilation_context::DXBWithSharedValues,
    prelude::*,
    runtime::{
        Runtime,
        execution::{
            ExecutionError,
            execution_loop::{
                interrupts::{ExternalExecutionInterrupt, InterruptProvider},
                state::{ExecutionLoopState, RuntimeExecutionStack},
            },
        },
    },
    shared_values::SharedContainer,
};

use crate::values::core_values::endpoint::Endpoint;

#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    pub verbose: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutionCallerMetadata {
    pub endpoint: Endpoint,
}

impl ExecutionCallerMetadata {
    pub fn local_default() -> Self {
        Self {
            endpoint: Endpoint::LOCAL,
        }
    }
    pub fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

/// Input required to execute a DXB program.
#[derive(Debug)]
pub struct ExecutionInput {
    /// Options for execution.
    pub options: ExecutionOptions,
    /// Metadata about the caller of the execution
    pub caller_metadata: ExecutionCallerMetadata,
    /// The DXB program body containing raw bytecode.
    pub dxb: Vec<u8>,
    pub shared_values: Vec<SharedContainer>,
    /// For persisting execution state across multiple executions (e.g., for REPL scenarios).
    pub loop_state: Option<ExecutionLoopState>,
    pub runtime: Runtime,
}

impl ExecutionInput {
    pub fn new(
        dxb_with_shared_values: DXBWithSharedValues,
        caller_metadata: ExecutionCallerMetadata,
        options: ExecutionOptions,
        runtime: Runtime,
    ) -> Self {
        Self {
            options,
            caller_metadata,
            dxb: dxb_with_shared_values.dxb,
            shared_values: dxb_with_shared_values.shared_values,
            loop_state: None,
            runtime,
        }
    }

    pub fn new_with_loop_state(
        dxb_with_shared_values: DXBWithSharedValues,
        caller_metadata: ExecutionCallerMetadata,
        options: ExecutionOptions,
        runtime: Runtime,
        loop_state: Option<ExecutionLoopState>,
    ) -> Self {
        Self {
            options,
            caller_metadata,
            dxb: dxb_with_shared_values.dxb,
            shared_values: dxb_with_shared_values.shared_values,
            loop_state,
            runtime,
        }
    }

    pub fn new_with_stack(
        dxb_with_shared_values: DXBWithSharedValues,
        caller_metadata: ExecutionCallerMetadata,
        options: ExecutionOptions,
        runtime: Runtime,
        stack: RuntimeExecutionStack,
    ) -> Self {
        let state = ExecutionLoopState::new(
            dxb_with_shared_values.dxb.clone(), // FIXME avoid clone
            dxb_with_shared_values.shared_values,
            runtime.clone(),
            stack,
            caller_metadata.clone(),
        );
        Self {
            options,
            caller_metadata,
            dxb: dxb_with_shared_values.dxb,
            shared_values: vec![], // FIXME: shared_values where moved to ExecutionLoopState, is that okay?
            loop_state: Some(state),
            runtime,
        }
    }
    pub fn into_dxb_and_shared_values(self) -> DXBWithSharedValues {
        DXBWithSharedValues::new(self.dxb, self.shared_values)
    }

    pub fn execution_loop(
        mut self,
    ) -> (
        InterruptProvider,
        impl Iterator<Item = Result<ExternalExecutionInterrupt, ExecutionError>>,
    ) {
        // use execution iterator if one already exists from previous execution
        let mut loop_state =
            if let Some(existing_loop_state) = self.loop_state.take() {
                // update dxb so that instruction iterator can continue with next instructions
                *existing_loop_state.dxb_body.borrow_mut() = self.dxb;
                existing_loop_state
            }
            // otherwise start a new execution loop
            else {
                ExecutionLoopState::new(
                    self.dxb,
                    self.shared_values,
                    self.runtime.clone(),
                    Default::default(),
                    self.caller_metadata.clone(),
                )
            };
        let interrupt_provider = loop_state.interrupt_provider.clone();

        // proxy the iterator, storing it back into state if interrupted to await more instructions
        let iterator = gen move {
            loop {
                let item = loop_state.iterator.next();
                if item.is_none() {
                    break;
                }
                let item = item.unwrap();

                match item {
                    Err(ExecutionError::IntermediateResultWithState(
                        box intermediate_result,
                        _,
                    )) => {
                        yield Err(
                            ExecutionError::intermediate_result_with_state(
                                intermediate_result,
                                Some(loop_state),
                            ),
                        );
                        break;
                    }
                    _ => yield item,
                }
            }
        };

        (interrupt_provider, iterator)
    }
}
