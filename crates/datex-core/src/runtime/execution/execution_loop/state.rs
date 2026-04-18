use crate::{
    runtime::{
        RuntimeInternal,
        execution::{
            ExecutionError,
            execution_loop::{
                ExternalExecutionInterrupt, execution_loop,
                interrupts::InterruptProvider,
            },
        },
    },
    shared_values::shared_containers::observers::TransceiverId,
    values::value_container::ValueContainer,
};
use core::{cell::RefCell, fmt::Debug};
use crate::global::protocol_structures::injected_values::{InjectedValueDeclaration, InjectedValueType, SharedInjectedValueType};
use crate::global::protocol_structures::instruction_data::StackIndex;
use crate::prelude::*;
use crate::runtime::execution::execution_input::ExecutionCallerMetadata;
use crate::runtime::execution::macros::yield_unwrap;
use crate::values::borrowed_value_container::BorrowedValueContainer;

pub struct ExecutionLoopState {
    pub iterator: Box<
        dyn Iterator<Item = Result<ExternalExecutionInterrupt, ExecutionError>>,
    >,
    pub dxb_body: Rc<RefCell<Vec<u8>>>,
    pub(crate) interrupt_provider: InterruptProvider,
}
impl ExecutionLoopState {
    pub fn new(
        dxb_body: Vec<u8>,
        runtime: Rc<RuntimeInternal>,
        stack: RuntimeExecutionStack,
        caller_metadata: ExecutionCallerMetadata,
    ) -> Self {
        let state = RuntimeExecutionState {
            runtime_internal: runtime.clone(),
            source_id: 0, // TODO #640: set proper source ID
            stack,
            caller_metadata,
        };
        // TODO #641: optimize, don't clone the whole DXB body every time here
        let dxb_rc = Rc::new(RefCell::new(dxb_body.to_vec()));
        let interrupt_provider = InterruptProvider::new();
        ExecutionLoopState {
            dxb_body: dxb_rc.clone(),
            iterator: Box::new(execution_loop(
                state,
                dxb_rc,
                interrupt_provider.clone(),
            )),
            interrupt_provider,
        }
    }
}

impl Debug for ExecutionLoopState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ExecutionIterator")
            .field("dxb_body_length", &self.dxb_body.borrow().len())
            .finish()
    }
}

#[derive(Debug)]
pub struct RuntimeExecutionState {
    /// Local memory stack for current execution context.
    pub stack: RuntimeExecutionStack,
    pub runtime_internal: Rc<RuntimeInternal>,
    pub source_id: TransceiverId,
    pub caller_metadata: ExecutionCallerMetadata
}

#[derive(Debug, Default)]
pub struct RuntimeExecutionStack {
    pub values: Vec<Option<ValueContainer>>,
}

impl RuntimeExecutionStack {
    /// Pushes a value to the stack
    pub(crate) fn push(
        &mut self,
        value: ValueContainer,
    ) {
        self.values.push(Some(value));
    }


    /// Pushes multiple values to the stack
    pub(crate) fn push_multiple(
        &mut self,
        values: Vec<ValueContainer>,
    ) {
        self.values.extend(values.into_iter().map(Some));
    }

    /// Takes a stack value by its index and returns its value.
    /// If the stack value is not allocated or the index is out of bounds, it returns an error.
    pub(crate) fn take_stack_value(
        &mut self,
        index: StackIndex,
    ) -> Result<ValueContainer, ExecutionError> {
        if let Some(stack_value) = self.values.get_mut(index.0 as usize) {
            stack_value.take().ok_or_else(|| ExecutionError::StackValueNotAllocated(index))
        }
        else {
            Err(ExecutionError::StackOutOfBoundsAccess(index))
        }
    }

    /// Sets the value of a stack index, returning the previous value if it existed.
    /// If the stack value is not allocated, it returns an error.
    pub(crate) fn set_stack_value(
        &mut self,
        index: StackIndex,
        value: ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        if let Some(stack_value) = self.values.get_mut(index.0 as usize) {
            Ok(stack_value.replace(value))
        }
        else {
            Err(ExecutionError::StackOutOfBoundsAccess(index))
        }
    }

    /// Retrieves a reference to the value of a stack value by its address.
    /// If the stack value is not allocated, it returns an error.
    pub(crate) fn get_stack_value(
        &self,
        index: StackIndex,
    ) -> Result<&ValueContainer, ExecutionError> {
        if let Some(stack_value) = self.values.get(index.0 as usize) {
            stack_value.as_ref().ok_or_else(|| ExecutionError::StackValueNotAllocated(index))
        }
        else {
            Err(ExecutionError::StackOutOfBoundsAccess(index))
        }
    }

    /// Retrieves a mutable reference to the stack value by its index.
    /// If the stack value is not allocated, it returns an error.
    pub(crate) fn get_stack_value_mut(
        &mut self,
        index: StackIndex,
    ) -> Result<&mut ValueContainer, ExecutionError> {
        if let Some(stack_value) = self.values.get_mut(index.0 as usize) {
            stack_value.as_mut().ok_or_else(|| ExecutionError::StackValueNotAllocated(index))
        }
        else {
            Err(ExecutionError::StackOutOfBoundsAccess(index))
        }
    }

    /// Resolves a list of injected values to actual values on the stack
    pub fn resolve_injected_values(&mut self, injected_values: &[InjectedValueDeclaration]) -> Result<Vec<BorrowedValueContainer>, ExecutionError> {
        let mut moved: Vec<Option<_>> = vec![None; injected_values.len()];

        // perform all mutable operations (removing moved shared values)
        for (i, InjectedValueDeclaration {index, ty}) in injected_values.iter().enumerate() {
            if matches!(ty, InjectedValueType::Shared(SharedInjectedValueType::Move)) {
                moved[i] = Some(self.take_stack_value(*index)?);
            }
        }

        // collect all values
        let mut resolved_values = Vec::with_capacity(injected_values.len());
        for (i, InjectedValueDeclaration {index, ty}) in injected_values.iter().enumerate() {
            resolved_values.push(match ty {
                InjectedValueType::Shared(SharedInjectedValueType::Move) => {
                    match moved[i].take().unwrap() {
                        ValueContainer::Shared(shared) => BorrowedValueContainer::Shared(shared),
                        ValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue)
                    }
                }
                _ => {
                    match self.get_stack_value(*index)? {
                        ValueContainer::Shared(shared) => BorrowedValueContainer::Shared(shared.clone()),
                        ValueContainer::Local(value) => BorrowedValueContainer::Local(value),
                    }
                }
            });
        }

        Ok(resolved_values)
    }
}
