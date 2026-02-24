use crate::runtime::{
    RuntimeInternal,
    execution::{
        ExecutionError,
        execution_loop::{
            interrupts::{ExternalExecutionInterrupt, InterruptProvider},
            state::{ExecutionLoopState, RuntimeExecutionSlots},
        },
    },
};

use crate::prelude::*;
#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    pub verbose: bool,
}

/// Input required to execute a DXB program.
#[derive(Debug, Default)]
pub struct ExecutionInput<'a> {
    /// Options for execution.
    pub options: ExecutionOptions,
    /// The DXB program body containing raw bytecode.
    pub dxb_body: &'a [u8],
    /// For persisting execution state across multiple executions (e.g., for REPL scenarios).
    pub loop_state: Option<ExecutionLoopState>,
    pub runtime: Option<Rc<RuntimeInternal>>,
}

impl<'a> ExecutionInput<'a> {
    pub fn new(
        dxb_body: &'a [u8],
        options: ExecutionOptions,
        runtime: Option<Rc<RuntimeInternal>>,
    ) -> Self {
        Self {
            options,
            dxb_body,
            loop_state: None,
            runtime,
        }
    }
    pub fn new_with_slots(
        dxb_body: &'a [u8],
        options: ExecutionOptions,
        runtime: Option<Rc<RuntimeInternal>>,
        slots: RuntimeExecutionSlots,
    ) -> Self {
        let state =
            ExecutionLoopState::new(dxb_body.to_vec(), runtime.clone(), slots);
        Self {
            options,
            dxb_body,
            loop_state: Some(state),
            runtime,
        }
    }

    pub fn execution_loop(
        mut self,
    ) -> (
        InterruptProvider,
        impl Iterator<Item = Result<ExternalExecutionInterrupt, ExecutionError>>,
    ) {
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
            ExecutionLoopState::new(
                self.dxb_body.to_vec(),
                self.runtime.clone(),
                Default::default(),
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
