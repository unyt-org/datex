use crate::{
    core_compiler::{
        core_compilation_context::{CompileInput, DXBWithSharedValues},
        injected_values::compile_injected_values,
    },
    dxb_parser::{
        body::{DXBParserError, iterate_instructions},
        instruction_collector::{
            CollectedResults, CollectionResultsPopper, FullOrPartialResult,
            InstructionCollector, LastUnboundedResultCollector,
            ResultCollector, StatementResultCollectionStrategy,
        },
    },
    global::{
        operators::{BinaryOperator, ComparisonOperator, UnaryOperator},
        protocol_structures::{
            instruction_data::{
                ApplyData, Float32Data, Float64Data, FloatAsInt16Data,
                FloatAsInt32Data, InstantData, ModifyStackValue,
                ShortStatementsData, ShortTextData, StatementsData,
                TaggedValue, TextData, UnboundedStatementsData,
            },
            instructions::{Instruction, NestedInstructionResolutionStrategy},
            regular_instructions::RegularInstruction,
            type_instructions::TypeInstruction,
        },
    },
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    runtime::{
        Runtime,
        cache::shared_values_cache::{
            CacheValueRetrievalError, ValueNotFoundInCacheError,
        },
        execution::{
            ExecutionError, InvalidProgramError,
            execution_loop::{
                collected_execution_result::CollectedExecutionResult,
                internal_slots::get_root_property,
                interrupts::{
                    ExecutionInterrupt, ExternalExecutionInterrupt,
                    InterruptProvider, InterruptResult,
                },
                runtime_value::RuntimeValue,
                state::RuntimeExecutionState,
            },
            macros::{
                interrupt, interrupt_with_maybe_value, interrupt_with_value,
            },
        },
    },
    shared_values::{
        PointerAddress, ReferenceMutability, ReferencedSharedContainer,
        RemotePointerAddress, SharedContainer, SharedContainerMutability,
        SharedContainerOwnership,
        base_shared_value_container::BaseSharedValueContainer,
    },
    types::{
        r#type::Type,
        type_definition::{
            TypeDefinition, impl_type::ImplTypeDefinition,
            range::RangeTypeDefinition, tagged_type::TaggedTypeDefinition,
        },
        type_definition_with_metadata::TypeDefinitionWithMetadata,
    },
    value_updates::{
        update_data::DeleteEntryUpdateData, update_handler::UpdateHandler,
    },
    values::{
        core_value::CoreValue,
        core_values::{
            callable::{Callable, CallableBody, CoreStub},
            decimal::{Decimal, typed_decimal::TypedDecimal},
            endpoint::Endpoint,
            integer::{Integer, typed_integer::TypedInteger},
            list::List,
            map::{Map, MapKey},
        },
        value::Value,
        value_container::{
            ValueContainer, error::ValueError, value_key::ValueKey,
        },
    },
};
use core::{cell::RefCell, ops::DerefMut};

impl
    CollectionResultsPopper<
        CollectedExecutionResult,
        Option<RuntimeValue>,
        MapKey,
        ValueContainer,
        Type,
        TypeDefinition,
    > for CollectedResults<CollectedExecutionResult>
{
    fn try_extract_type_definition(
        result: CollectedExecutionResult,
    ) -> Option<TypeDefinition> {
        match result {
            CollectedExecutionResult::TypeDefinition(ty) => Some(*ty),
            _ => None,
        }
    }
    fn try_extract_value(
        result: CollectedExecutionResult,
    ) -> Option<Option<RuntimeValue>> {
        match result {
            CollectedExecutionResult::Value(box val) => Some(val),
            _ => None,
        }
    }

    fn try_extract_type(result: CollectedExecutionResult) -> Option<Type> {
        match result {
            CollectedExecutionResult::Type(ty) => Some(*ty),
            _ => None,
        }
    }

    fn try_extract_key_value_pair(
        result: CollectedExecutionResult,
    ) -> Option<(MapKey, ValueContainer)> {
        match result {
            CollectedExecutionResult::KeyValuePair(box (key, value)) => {
                Some((key, value))
            }
            _ => None,
        }
    }
}

impl CollectedResults<CollectedExecutionResult> {
    /// Collect multiple owned value containers
    pub fn try_collect_value_containers(
        mut self,
        state: &mut RuntimeExecutionState,
    ) -> Result<Vec<ValueContainer>, ExecutionError> {
        let count = self.len();
        let mut expressions = Vec::with_capacity(count);
        for _ in 0..count {
            expressions.push(self.try_pop_value_container(state)?);
        }
        expressions.reverse();
        Ok(expressions)
    }

    /// Pops a runtime value result, returning an error if none exists
    pub fn try_pop_runtime_value(
        &mut self,
    ) -> Result<RuntimeValue, ExecutionError> {
        self.pop_value().ok_or(ExecutionError::invalid_program(
            InvalidProgramError::ExpectedValue,
        ))
    }

    /// Pops an owned value container, returning an error if none exists.
    /// If the value is a slot address, it is resolved to a value container and the slot is removed from the runtime state.
    pub fn try_pop_value_container(
        &mut self,
        state: &mut RuntimeExecutionState,
    ) -> Result<ValueContainer, ExecutionError> {
        self.try_pop_runtime_value()?.into_value_container(state)
    }

    /// Pops a key-value pair result, returning an error if none exists
    pub fn try_collect_key_value_pair(
        mut self,
    ) -> Result<Vec<(MapKey, ValueContainer)>, ExecutionError> {
        let count = self.len();
        let mut pairs = Vec::with_capacity(count);
        for _ in 0..count {
            let (key, value) = self.pop_key_value_pair();
            pairs.push((key, value));
        }
        pairs.reverse();
        Ok(pairs)
    }
}
