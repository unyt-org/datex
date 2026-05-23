use alloc::rc::Rc;
use core::cell::RefCell;

use crate::runtime::{
    Runtime,
    execution::{
        ExecutionError,
        execution_loop::{
            interrupts::{ExternalExecutionInterrupt, InterruptProvider},
            state::{ExecutionLoopState, RuntimeExecutionStack},
        },
    },
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
pub struct ExecutionInput<'a> {
    /// Options for execution.
    pub options: ExecutionOptions,
    /// Metadata about the caller of the execution
    pub caller_metadata: ExecutionCallerMetadata,
    /// The DXB program body containing raw bytecode.
    pub dxb_body: &'a [u8],
    /// For persisting execution state across multiple executions (e.g., for REPL scenarios).
    pub loop_state: Option<ExecutionLoopState>,
    pub runtime: Runtime,
    /// Capture for the final stack after execution completes.
    pub(crate) stack_capture: Option<Rc<RefCell<Option<RuntimeExecutionStack>>>>,
}

impl<'a> ExecutionInput<'a> {
    pub fn new(
        dxb_body: &'a [u8],
        caller_metadata: ExecutionCallerMetadata,
        options: ExecutionOptions,
        runtime: Runtime,
    ) -> Self {
        Self {
            options,
            caller_metadata,
            dxb_body,
            loop_state: None,
            runtime,
            stack_capture: None,
        }
    }
    pub fn new_with_stack(
        dxb_body: &'a [u8],
        caller_metadata: ExecutionCallerMetadata,
        options: ExecutionOptions,
        runtime: Runtime,
        stack: RuntimeExecutionStack,
    ) -> Self {
        let state = ExecutionLoopState::new(
            dxb_body.to_vec(),
            runtime.clone(),
            stack,
            caller_metadata.clone(),
        );
        Self {
            options,
            caller_metadata,
            dxb_body,
            loop_state: Some(state),
            runtime,
            stack_capture: None,
        }
    }

    pub fn new_with_stack_capture(
        dxb_body: &'a [u8],
        caller_metadata: ExecutionCallerMetadata,
        options: ExecutionOptions,
        runtime: Runtime,
        stack: RuntimeExecutionStack,
        stack_capture: Rc<RefCell<Option<RuntimeExecutionStack>>>,
    ) -> Self {
        let state = ExecutionLoopState::new_with_stack_capture(
            dxb_body.to_vec(),
            runtime.clone(),
            stack,
            caller_metadata.clone(),
            Some(stack_capture.clone()),
        );
        Self {
            options,
            caller_metadata,
            dxb_body,
            loop_state: Some(state),
            runtime,
            stack_capture: Some(stack_capture),
        }
    }

    pub fn execution_loop(
        mut self,
    ) -> (
        InterruptProvider,
        impl Iterator<Item = Result<ExternalExecutionInterrupt, ExecutionError>>,
    ) {
        let stack_capture = self.stack_capture.take();
        // use execution iterator if one already exists from previous execution
        let mut loop_state = if let Some(existing_loop_state) =
            self.loop_state.take()
        {
            // update dxb so that instruction iterator can continue with next instructions
            *existing_loop_state.dxb_body.borrow_mut() = self.dxb_body.to_vec();
            existing_loop_state
        }
        // otherwise start a new execution loop
        else {
            ExecutionLoopState::new_with_stack_capture(
                self.dxb_body.to_vec(),
                self.runtime.clone(),
                Default::default(),
                self.caller_metadata.clone(),
                stack_capture,
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
                        intermediate_result,
                        _,
                    )) => {
                        yield Err(ExecutionError::IntermediateResultWithState(
                            intermediate_result,
                            Some(loop_state),
                        ));
                        break;
                    }
                    _ => yield item,
                }
            }
        };

        (interrupt_provider, iterator)
    }
}
