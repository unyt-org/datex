use binrw::io::Write;
use binrw::io::Cursor;
use crate::core_compiler::core_compilation_context::CoreCompilationContext;
use crate::core_compiler::value_compiler::{append_instruction_code, append_instruction_code_new, append_perform_moves, append_shared_container, append_statements_preamble};
use crate::global::instruction_codes::InstructionCode;
use crate::global::protocol_structures::external_slot_type::{ExternalSlotType, SharedSlotType};
use crate::global::protocol_structures::instruction_data::{InstructionBlockData, SlotAddress};
use crate::runtime::execution::ExecutionError;
use crate::shared_values::shared_container::SharedContainer;
use crate::utils::buffers::{append_u32, append_u8};
use crate::prelude::*;
use crate::values::borrowed_value_container::BorrowedValueContainer;

/// Compiles a remote execution block into a bytecode buffer, with the given instruction block metadata and injected values
/// which can then be sent to another endpoint
/// Refs are injected as separate slots at the top, e.g.:
/// #0 = GET_REF x;
/// #1 = GET_REF_MUT y;
///
/// Moves are performed in a single perform_move instruction:
/// #2 = PERFORM_MOVE a, b, ...;
/// #3 = #2.0
/// #4 = #2.1
pub fn compile_remote_execution_block(
    exec_block_data: InstructionBlockData,
    slot_values: Vec<BorrowedValueContainer>,
) -> Result<(Vec<u8>, Vec<SharedContainer>), ExecutionError> {

    if exec_block_data
        .injected_slots
        .len() != slot_values.len() {
        unreachable!(); // length must always match
    }

    let moved_pointers_slot_index = exec_block_data
        .injected_slots
        .len() as u32;

    // one slot assignment statement for each slot + original instructions block
    let mut preamble_statements_count = slot_values.len() as u32;

    let (mut slot_preamble, moved_owned_containers) = compile_preamble(moved_pointers_slot_index, exec_block_data.clone(), slot_values)?;

    // if there are any moved pointers, we need to compile the preform_move instruction and allocate a slot for the moved pointers
    if !moved_owned_containers.is_empty() {
        // + 1 statement for perform move
        preamble_statements_count += 1;
        let move_preamble = compile_preform_move_preamble(moved_pointers_slot_index, &moved_owned_containers.iter().collect::<Vec<&SharedContainer>>());
        // prepend before slot_preamble
        slot_preamble = [move_preamble, slot_preamble].concat();
    }

    let final_buffer = if preamble_statements_count > 0 {
        let mut final_buffer = Cursor::new(Vec::with_capacity(6 + exec_block_data.body.len() + slot_preamble.len()));
        append_statements_preamble(&mut final_buffer, preamble_statements_count as usize + 1, false);
        final_buffer.write_all(&slot_preamble).unwrap();
        final_buffer.write_all(
            &exec_block_data.body,
        ).unwrap();
        final_buffer.into_inner()
    }
    else {
        exec_block_data.body
    };

    Ok((final_buffer, moved_owned_containers))
}

fn compile_preamble(
    moved_pointers_slot_index: u32,
    exec_block_data: InstructionBlockData,
    slot_values: Vec<BorrowedValueContainer>,
) -> Result<(Vec<u8>, Vec<SharedContainer>), ExecutionError> {
    let mut context = CoreCompilationContext::new(Vec::new(), SlotAddress(0));

    let mut moved_pointers: Vec<SharedContainer> = vec![];

    // build dxb
    for (slot_addr, ((_, external_slot_type), slot_value)) in exec_block_data
        .injected_slots
        .into_iter()
        .zip(slot_values.into_iter())
        .enumerate()
    {
        context.cursor_mut().write_all(&[InstructionCode::ALLOCATE_SLOT as u8]).unwrap();
        append_u32(context.cursor_mut(), slot_addr as u32);
        match external_slot_type {
            ExternalSlotType::Local(_) => {
                todo!()
            },
            ExternalSlotType::Shared(shared_slot_type) => {

                let shared_container = match shared_slot_type {
                    SharedSlotType::Move => {
                        // get moved value from moved_pointers_slot
                        let index = moved_pointers.len() as u32;

                        match slot_value {
                            BorrowedValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
                            BorrowedValueContainer::Shared(shared_container) => {
                                shared_container.assert_owned().map_err(|_| ExecutionError::ExpectedOwnedSharedValue)?;
                                moved_pointers.push(shared_container);
                                append_instruction_code_new(context.cursor_mut(), InstructionCode::TAKE_PROPERTY_INDEX);
                                append_u32(context.cursor_mut(), index);
                                append_instruction_code_new(context.cursor_mut(), InstructionCode::CLONE_SLOT);
                                append_u32(context.cursor_mut(), moved_pointers_slot_index);
                                continue;
                            }
                        }
                    },
                    SharedSlotType::Ref => match slot_value {
                        BorrowedValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
                        BorrowedValueContainer::Shared(shared_container) => {
                            shared_container.derive_reference()
                        }
                    }
                    SharedSlotType::RefMut => match slot_value {
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
    let mut context = CoreCompilationContext::new(Vec::new(), SlotAddress(0));
    context.cursor_mut().write_all(&[InstructionCode::ALLOCATE_SLOT as u8]).unwrap();

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
    use crate::global::protocol_structures::external_slot_type::{ExternalSlotType, SharedSlotType};
    use crate::global::protocol_structures::instruction_data::InstructionBlockData;
    use crate::runtime::execution::execution_loop::remote_execution_blocks::compile_remote_execution_block;
    use crate::shared_values::pointer::{OwnedPointer};
    use crate::shared_values::shared_container::SharedContainer;
    use crate::prelude::*;
    use crate::values::borrowed_value_container::BorrowedValueContainer;

    #[test]
    fn remote_execution_no_injected_values() {
        let exec_block_data = InstructionBlockData {
            injected_slot_count: 0,
            length: 1,
            injected_slots: vec![],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_remote_execution_block(exec_block_data, vec![]).unwrap().0;
        assert_eq!(res, vec![InstructionCode::NULL as u8]);
    }

    #[test]
    fn remote_execution_with_injected_ref_value() {
        let shared_value = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_immut(42, OwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_slot_count: 1,
            length: 1,
            injected_slots: vec![(0, ExternalSlotType::Shared(SharedSlotType::Ref))],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_remote_execution_block(exec_block_data, vec![shared_value]).unwrap().0;
        // should allocate slot and then compile the shared value into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS as u8,
                2,
                0,
                InstructionCode::ALLOCATE_SLOT as u8,
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
        let shared_value1 = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_immut(42, OwnedPointer::NULL));
        let shared_value2 = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_mut(100, OwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_slot_count: 2,
            length: 1,
            injected_slots: vec![
                (0, ExternalSlotType::Shared(SharedSlotType::Ref)),
                (1, ExternalSlotType::Shared(SharedSlotType::RefMut)),
            ],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_remote_execution_block(exec_block_data, vec![shared_value1, shared_value2]).unwrap().0;
        // should allocate slots and then compile the shared values into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS as u8,
                3,
                0,
                InstructionCode::ALLOCATE_SLOT as u8,
                0, 0, 0, 0, // slot address of first value
                // compiled shared reference for first value
                InstructionCode::SHARED_REF_WITH_VALUE as u8,
                0, 0, 0, 0, 0, // address of the first shared value
                0, // immutable ref
                0, // immutable container
                InstructionCode::INT_32 as u8,
                42, 0, 0, 0, // value of the first shared integer
                InstructionCode::ALLOCATE_SLOT as u8,
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
        let shared_value = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_immut(42, OwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_slot_count: 1,
            length: 1,
            injected_slots: vec![(0, ExternalSlotType::Shared(SharedSlotType::Move))],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_remote_execution_block(exec_block_data, vec![shared_value]).unwrap().0;
        // should allocate slot and then compile the shared value into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS as u8,
                3,
                0,
                InstructionCode::ALLOCATE_SLOT as u8,
                1, 0, 0, 0, // slot address of moved pointers
                // compiled shared moves
                InstructionCode::PERFORM_MOVE as u8,
                1, 0, 0, 0, // number of moves (1)
                0, // immutable
                0, 0, 0, 0, 0, // pointer address (assuming the shared container is stored at address 1)
                InstructionCode::ALLOCATE_SLOT as u8,
                0, 0, 0, 0, // slot address
                InstructionCode::TAKE_PROPERTY_INDEX as u8,
                0, 0, 0, 0, // index of the moved pointer
                InstructionCode::CLONE_SLOT as u8,
                1, 0, 0, 0, // slot address of the moved pointers
                InstructionCode::NULL as u8, // body
            ]
        );
    }

    #[test]
    fn remote_execution_moved_value_and_ref() {
        let shared_value1 = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_immut(42, OwnedPointer::NULL));
        let shared_value2 = BorrowedValueContainer::Shared(SharedContainer::boxed_owned_mut(100, OwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_slot_count: 2,
            length: 1,
            injected_slots: vec![
                (0, ExternalSlotType::Shared(SharedSlotType::Move)),
                (1, ExternalSlotType::Shared(SharedSlotType::Ref)),
            ],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_remote_execution_block(exec_block_data, vec![shared_value1, shared_value2]).unwrap().0;
        // should allocate slots and then compile the shared values into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS as u8,
                4,
                0,
                InstructionCode::ALLOCATE_SLOT as u8,
                2, 0, 0, 0, // slot address of moved pointers
                // compiled shared moves
                InstructionCode::PERFORM_MOVE as u8,
                1, 0, 0, 0, // number of moves (1)
                0, // immmut
                0, 0, 0, 0, 0, // pointer address (assuming the first shared container is stored at address 0)

                InstructionCode::ALLOCATE_SLOT as u8,
                0, 0, 0, 0, // slot address of first value (moved)
                InstructionCode::TAKE_PROPERTY_INDEX as u8,
                0, 0, 0, 0, // index of the moved pointer
                InstructionCode::CLONE_SLOT as u8,
                2, 0, 0, 0, // slot address of the moved pointers

                InstructionCode::ALLOCATE_SLOT as u8,
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