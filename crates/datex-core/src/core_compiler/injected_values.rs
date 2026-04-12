use core::cell::RefCell;
use binrw::io::Write;
use crate::core_compiler::core_compilation_context::CoreCompilationContext;
use crate::core_compiler::value_compiler::{append_instruction_code, append_instruction_code_new, append_perform_moves, append_regular_instruction, append_shared_container, append_statements_preamble, append_value, SharedValueCompilationError};
use crate::global::instruction_codes::InstructionCode;
use crate::global::protocol_structures::injected_values::{InjectedValueDeclaration, InjectedValueType, SharedInjectedValueType};
use crate::global::protocol_structures::instruction_data::{InstructionBlockData, PerformMove, RawLocalPointerAddress, StackIndex};
use crate::global::protocol_structures::regular_instructions::RegularInstruction;
use crate::runtime::execution::ExecutionError;
use crate::shared_values::shared_container::{SharedContainer, SharedContainerInner};
use crate::utils::buffers::{append_u32, append_u8};
use crate::prelude::*;
use crate::values::borrowed_value_container::BorrowedValueContainer;
pub fn compile_injected_values(
    instruction_block_data: InstructionBlockData,
    injected_values: Vec<BorrowedValueContainer>,
) -> Result<(Vec<u8>, Vec<SharedContainer>), SharedValueCompilationError> {
    let mut context = CoreCompilationContext::new(Vec::new());
    compile_injected_values_with_context(
        &mut context,
        instruction_block_data,
        injected_values
    )?;
    Ok(context.into_buffer_and_moved_values())
}

/// Prepends injected values to an instruction block
/// This is used for remote execution blocks and function bodies.
///
/// #stack ..= (
/// ///    Set($1, $2,)
///    #0 = MOVE (1,2,34);
///
///    -----
///    #parent = SHARED_REF 1;
///    #child = {p: #parent}
///    #parent.c = #child;
///    #3 = #0[1]
///    -----
///
///
///    [
///      #stack[1],
///       parent {
///          x: parent,
///          y: #stack[2]
///       },
///       #stack[3],
///       {
///         x: 1,
///       }
///    ]
///
/// )
///
/// compile_injected_values ()
/// for chilren(compile_injected_values)
/// Set.add(shared)
/// Instruction::
/// x;
pub fn compile_injected_values_with_context(
    compilation_context: &mut CoreCompilationContext,
    instruction_block_data: InstructionBlockData,
    injected_values: Vec<BorrowedValueContainer>,
) -> Result<(), SharedValueCompilationError> {

    if instruction_block_data
        .injected_values
        .len() != injected_values.len() {
        unreachable!(); // length must always match
    }

    for injected_value in injected_values {
        match injected_value {
            BorrowedValueContainer::Local(local_value) => {
                append_value(compilation_context, local_value)?;
            }
            BorrowedValueContainer::Shared(shared_value) => {
                append_shared_container(compilation_context, &shared_value, false)?;
            }
        }
    }

    // compile preamble

    // ?
    Ok(())
}

pub fn compile_shared_value_preamble(compilation_context: &mut CoreCompilationContext) {
    let shared_value_tracking = &compilation_context.shared_value_tracking;
    let cursor = &mut compilation_context.cursor;

    let moved_ptr_addresses = shared_value_tracking.get_moved_shared_addresses();

    append_regular_instruction(
        cursor,
        RegularInstruction::PushToStack,
    );

    // push NULL to stack#1 if no moves
    if moved_ptr_addresses.is_empty() {
        append_regular_instruction(
            cursor,
            RegularInstruction::Null,
        )
    }
    // push moves
    else {
        append_regular_instruction(
            cursor,
            RegularInstruction::PerformMove(PerformMove {
                pointer_count: moved_ptr_addresses.len() as u32,
                pointers: moved_ptr_addresses
                    .iter()
                    .map(|shared_container| {
                        (
                            0, // TODO: insert value or not?
                            RawLocalPointerAddress {bytes: shared_container.address }
                        )
                    })
                    .collect(),
            })
        );
    }
}



fn compile_preamble(
    moved_pointers_slot_index: u32,
    exec_block_data: InstructionBlockData,
    slot_values: Vec<BorrowedValueContainer>,
) -> Result<(Vec<u8>, Vec<SharedContainer>), ExecutionError> {
    let mut context = CoreCompilationContext::new(Vec::new());

    let mut moved_pointers: Vec<SharedContainer> = vec![];

    // build dxb
    for (slot_addr, (InjectedValueDeclaration {ty: external_slot_type, ..}, slot_value)) in exec_block_data
        .injected_values
        .into_iter()
        .zip(slot_values.into_iter())
        .enumerate()
    {
        context.cursor_mut().write_all(&[InstructionCode::PUSH_TO_STACK as u8]).unwrap();
        append_u32(context.cursor_mut(), slot_addr as u32);
        match external_slot_type {
            InjectedValueType::Local(_) => {
                todo!()
            },
            InjectedValueType::Shared(shared_slot_type) => {

                let shared_container = match shared_slot_type {
                    SharedInjectedValueType::Move => {
                        // get moved value from moved_pointers_slot
                        let index = moved_pointers.len() as u32;

                        match slot_value {
                            BorrowedValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
                            BorrowedValueContainer::Shared(shared_container) => {
                                shared_container.assert_owned().map_err(|_| ExecutionError::ExpectedOwnedSharedValue)?;
                                moved_pointers.push(shared_container);
                                append_instruction_code_new(context.cursor_mut(), InstructionCode::TAKE_PROPERTY_INDEX);
                                append_u32(context.cursor_mut(), index);
                                append_instruction_code_new(context.cursor_mut(), InstructionCode::CLONE_STACK_VALUE);
                                append_u32(context.cursor_mut(), moved_pointers_slot_index);
                                continue;
                            }
                        }
                    },
                    SharedInjectedValueType::Ref => match slot_value {
                        BorrowedValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
                        BorrowedValueContainer::Shared(shared_container) => {
                            shared_container.derive_reference()
                        }
                    }
                    SharedInjectedValueType::RefMut => match slot_value {
                        BorrowedValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
                        BorrowedValueContainer::Shared(shared_container) => {
                            shared_container.try_derive_mutable_reference()
                                .map_err(|_| ExecutionError::MutableReferenceToNonMutableValue)?
                        }
                    }
                };

                append_shared_container(
                    &mut context,
                    &shared_container,
                    true
                ).unwrap();
            }
        }
    }

    Ok((
        context.into_buffer(),
        moved_pointers
    ))
}

fn compile_preform_move_preamble(
    moved_pointers_slot_index: u32,
    moved_pointers: &[&SharedContainer]
) -> Vec<u8> {
    let mut context = CoreCompilationContext::new(Vec::new());
    context.cursor_mut().write_all(&[InstructionCode::PUSH_TO_STACK as u8]).unwrap();

    append_u32(context.cursor_mut(), moved_pointers_slot_index);

    append_perform_moves(
        &mut context,
        moved_pointers
    ).unwrap(); // we already ensured that all moved pointers are owned local shared containers, so this should never fail

    context.into_buffer()
}

#[cfg(test)]
mod tests {
    use crate::global::instruction_codes::InstructionCode;
    use crate::global::protocol_structures::injected_values::{InjectedValueDeclaration, InjectedValueType, SharedInjectedValueType};
    use crate::global::protocol_structures::instruction_data::{InstructionBlockData, StackIndex};
    use crate::core_compiler::injected_values::compile_injected_values;
    use crate::shared_values::pointer::{EndpointOwnedPointer};
    use crate::shared_values::shared_container::SharedContainer;
    use crate::prelude::*;
    use crate::values::borrowed_value_container::BorrowedValueContainer;

    #[test]
    fn remote_execution_no_injected_values() {
        let exec_block_data = InstructionBlockData {
            injected_value_count: 0,
            length: 1,
            injected_values: vec![],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_injected_values(exec_block_data, vec![]).unwrap().0;
        assert_eq!(res, vec![InstructionCode::NULL as u8]);
    }

    #[test]
    fn remote_execution_with_injected_ref_value() {
        let shared_value = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_immut(42, EndpointOwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_value_count: 1,
            length: 1,
            injected_values: vec![InjectedValueDeclaration {index: StackIndex(0), ty: InjectedValueType::Shared(SharedInjectedValueType::Ref)}],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_injected_values(exec_block_data, vec![shared_value]).unwrap().0;
        // should allocate slot and then compile the shared value into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS as u8,
                2,
                0,
                InstructionCode::PUSH_TO_STACK as u8,
                0, 0, 0, 0, // slot address
                // compiled shared reference
                InstructionCode::SHARED_REF_WITH_VALUE as u8,
                0, 0, 0, 0, 0, // address of the shared value
                0, // immutable ref
                0, // immutable container
                InstructionCode::INT_32 as u8,
                42, 0, 0, 0, // value of the shared integer
                InstructionCode::NULL as u8, // body
            ]
        );
    }

    #[test]
    fn remote_execution_multiple_ref_values() {
        let shared_value1 = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_immut(42, EndpointOwnedPointer::NULL));
        let shared_value2 = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_mut(100, EndpointOwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_value_count: 2,
            length: 1,
            injected_values: vec![
                InjectedValueDeclaration {index: StackIndex(0), ty: InjectedValueType::Shared(SharedInjectedValueType::Ref)},
                InjectedValueDeclaration {index: StackIndex(1), ty: InjectedValueType::Shared(SharedInjectedValueType::RefMut)},
            ],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_injected_values(exec_block_data, vec![shared_value1, shared_value2]).unwrap().0;
        // should allocate slots and then compile the shared values into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS as u8,
                3,
                0,
                InstructionCode::PUSH_TO_STACK as u8,
                0, 0, 0, 0, // slot address of first value
                // compiled shared reference for first value
                InstructionCode::SHARED_REF_WITH_VALUE as u8,
                0, 0, 0, 0, 0, // address of the first shared value
                0, // immutable ref
                0, // immutable container
                InstructionCode::INT_32 as u8,
                42, 0, 0, 0, // value of the first shared integer
                InstructionCode::PUSH_TO_STACK as u8,
                1, 0, 0, 0, // slot address of second value
                // compiled shared mutable reference for second value
                InstructionCode::SHARED_REF_WITH_VALUE as u8,
                0, 0, 0, 0, 0, // address of the second shared value
                1, // mutable ref
                1, // mutable container
                InstructionCode::INT_32 as u8,
                100, 0, 0, 0, // value of the second shared integer
                InstructionCode::NULL as u8, // body
            ]
        );
    }


    #[test]
    fn remote_execution_with_injected_moved_value() {
        let shared_value = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_immut(42, EndpointOwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_value_count: 1,
            length: 1,
            injected_values: vec![InjectedValueDeclaration {index: StackIndex(0), ty: InjectedValueType::Shared(SharedInjectedValueType::Move)}],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_injected_values(exec_block_data, vec![shared_value]).unwrap().0;
        // should allocate slot and then compile the shared value into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS as u8,
                3,
                0,
                InstructionCode::PUSH_TO_STACK as u8,
                1, 0, 0, 0, // slot address of moved pointers
                // compiled shared moves
                InstructionCode::PERFORM_MOVE as u8,
                1, 0, 0, 0, // number of moves (1)
                0, // immutable
                0, 0, 0, 0, 0, // pointer address (assuming the shared container is stored at address 1)
                InstructionCode::PUSH_TO_STACK as u8,
                0, 0, 0, 0, // slot address
                InstructionCode::TAKE_PROPERTY_INDEX as u8,
                0, 0, 0, 0, // index of the moved pointer
                InstructionCode::CLONE_STACK_VALUE as u8,
                1, 0, 0, 0, // slot address of the moved pointers
                InstructionCode::NULL as u8, // body
            ]
        );
    }

    #[test]
    fn remote_execution_moved_value_and_ref() {
        let shared_value1 = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_immut(42, EndpointOwnedPointer::NULL));
        let shared_value2 = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_mut(100, EndpointOwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_value_count: 2,
            length: 1,
            injected_values: vec![
                InjectedValueDeclaration {index: StackIndex(0), ty: InjectedValueType::Shared(SharedInjectedValueType::Move)},
                InjectedValueDeclaration {index: StackIndex(1), ty: InjectedValueType::Shared(SharedInjectedValueType::Ref)},
            ],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_injected_values(exec_block_data, vec![shared_value1, shared_value2]).unwrap().0;
        // should allocate slots and then compile the shared values into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS as u8,
                4,
                0,
                InstructionCode::PUSH_TO_STACK as u8,
                2, 0, 0, 0, // slot address of moved pointers
                // compiled shared moves
                InstructionCode::PERFORM_MOVE as u8,
                1, 0, 0, 0, // number of moves (1)
                0, // immmut
                0, 0, 0, 0, 0, // pointer address (assuming the first shared container is stored at address 0)

                InstructionCode::PUSH_TO_STACK as u8,
                0, 0, 0, 0, // slot address of first value (moved)
                InstructionCode::TAKE_PROPERTY_INDEX as u8,
                0, 0, 0, 0, // index of the moved pointer
                InstructionCode::CLONE_STACK_VALUE as u8,
                2, 0, 0, 0, // slot address of the moved pointers

                InstructionCode::PUSH_TO_STACK as u8,
                1, 0, 0, 0, // slot address of second value
                // compiled shared reference for second value
                InstructionCode::SHARED_REF_WITH_VALUE as u8,
                0, 0, 0, 0, 0, // address of the second shared value
                0, // immutable ref
                1, // mutable value
                InstructionCode::INT_32 as u8,
                100, 0, 0, 0, // value of the second shared integer

                InstructionCode::NULL as u8, // body
            ]
        );
    }
}