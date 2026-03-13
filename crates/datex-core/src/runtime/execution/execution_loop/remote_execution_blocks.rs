use crate::core_compiler::value_compiler::{append_instruction_code, append_perform_moves, append_shared_container};
use crate::global::instruction_codes::InstructionCode;
use crate::global::protocol_structures::external_slot_type::{ExternalSlotType, SharedSlotType};
use crate::global::protocol_structures::instructions::InstructionBlockData;
use crate::runtime::execution::ExecutionError;
use crate::shared_values::shared_container::SharedContainer;
use crate::utils::buffers::append_u32;
use crate::values::value_container::ValueContainer;
use crate::prelude::*;

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
    slot_values: Vec<Cow<ValueContainer>>,
) -> Result<Vec<u8>, ExecutionError> {
    let mut buffer = Vec::with_capacity(256);

    let moved_pointers_slot_index = exec_block_data
        .injected_slots
        .len() as u32;

    let moved_pointers = compile_preamble(&mut buffer, moved_pointers_slot_index, exec_block_data.clone(), slot_values)?;

    // if there are any moved pointers, we need to compile the preform_move instruction and allocate a slot for the moved pointers
    if !moved_pointers.is_empty() {
        compile_preform_move_preamble(&mut buffer, moved_pointers_slot_index, &moved_pointers);
    }

    buffer.extend_from_slice(
        &exec_block_data.body,
    );

    Ok(buffer)
}

fn compile_preamble(
    buffer: &mut Vec<u8>,
    moved_pointers_slot_index: u32,
    exec_block_data: InstructionBlockData,
    slot_values: Vec<Cow<ValueContainer>>,
) -> Result<Vec<SharedContainer>, ExecutionError> {

    let mut moved_pointers: Vec<SharedContainer> = vec![];

    // build dxb
    for (slot_addr, ((_, external_slot_type), slot_value)) in exec_block_data
        .injected_slots
        .into_iter()
        .zip(slot_values.into_iter())
        .enumerate()
    {
        buffer.push(
            InstructionCode::ALLOCATE_SLOT
                as u8,
        );
        append_u32(buffer, slot_addr as u32);
        match external_slot_type {
            ExternalSlotType::Local(_) => {
                todo!()
            },
            ExternalSlotType::Shared(shared_slot_type) => {

                let shared_container = match shared_slot_type {
                    SharedSlotType::Move => {
                        // get moved value from moved_pointers_slot
                        let index = moved_pointers.len() as u32;

                        // this clones the slot value if it was not owned, leading to an ExpectedOwnedSharedValue error
                        // since the clone creates a ref instead of an owned value
                        let slot_value = slot_value.into_owned();
                        match slot_value {
                            ValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
                            ValueContainer::Shared(shared_container) => {
                                shared_container.assert_owned().map_err(|_| ExecutionError::ExpectedOwnedSharedValue)?;
                                moved_pointers.push(shared_container);
                                append_instruction_code(buffer, InstructionCode::GET_PROPERTY_INDEX);
                                append_u32(buffer, index);
                                append_instruction_code(buffer, InstructionCode::CLONE_SLOT);
                                append_u32(buffer, moved_pointers_slot_index);
                                continue;
                            }
                        }
                    },
                    SharedSlotType::Ref => match slot_value.as_ref() {
                        ValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
                        ValueContainer::Shared(shared_container) => {
                            shared_container.derive_reference()
                        }
                    }
                    SharedSlotType::RefMut => match slot_value.as_ref() {
                        ValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
                        ValueContainer::Shared(shared_container) => {
                            shared_container.try_derive_mutable_reference()
                                .map_err(|_| ExecutionError::MutableReferenceToNonMutableValue)?
                        }
                    }
                };

                append_shared_container(
                    buffer,
                    shared_container,
                    true
                ).map_err(|_| ExecutionError::ExpectedOwnedSharedValue)?;
            }
        }
    }

    Ok(moved_pointers)
}

fn compile_preform_move_preamble(
    buffer: &mut Vec<u8>,
    moved_pointers_slot_index: u32,
    moved_pointers: &[SharedContainer]
) {
    let mut pre_buffer = vec![
        InstructionCode::ALLOCATE_SLOT as u8,
    ];
    append_u32(&mut pre_buffer, moved_pointers_slot_index);

    append_perform_moves(
        &mut pre_buffer,
        moved_pointers
    ).unwrap(); // we already ensured that all moved pointers are owned local shared containers, so this should never fail

    // prepend pre_buffer to buffer
    buffer.splice(0..0, pre_buffer);
}

#[cfg(test)]
mod tests {
    use crate::global::instruction_codes::InstructionCode;
    use crate::global::protocol_structures::external_slot_type::{ExternalSlotType, SharedSlotType};
    use crate::global::protocol_structures::instructions::InstructionBlockData;
    use crate::runtime::execution::execution_loop::remote_execution_blocks::compile_remote_execution_block;
    use crate::shared_values::pointer::{OwnedPointer};
    use crate::shared_values::shared_container::SharedContainer;
    use crate::values::value_container::ValueContainer;
    use crate::prelude::*;

    #[test]
    fn remote_execution_no_injected_values() {
        let exec_block_data = InstructionBlockData {
            injected_slot_count: 0,
            length: 1,
            injected_slots: vec![],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_remote_execution_block(exec_block_data, vec![]).unwrap();
        assert_eq!(res, vec![InstructionCode::NULL as u8]);
    }

    #[test]
    fn remote_execution_with_injected_ref_value() {
        let shared_value = ValueContainer::Shared(SharedContainer::boxed_owned(42, OwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_slot_count: 1,
            length: 1,
            injected_slots: vec![(0, ExternalSlotType::Shared(SharedSlotType::Ref))],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_remote_execution_block(exec_block_data, vec![Cow::Owned(shared_value)]).unwrap();
        // should allocate slot and then compile the shared value into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::ALLOCATE_SLOT as u8,
                0, 0, 0, 0, // slot address
                // compiled shared reference
                InstructionCode::SHARED_REF as u8,
                1, // insert value
                0, 0, 0, 0, 0, // index of the shared value
                InstructionCode::INT_32 as u8,
                42, 0, 0, 0, // value of the shared integer
                InstructionCode::NULL as u8, // body
            ]
        );
    }

    #[test]
    fn remote_execution_multiple_ref_values() {
        let shared_value1 = ValueContainer::Shared(SharedContainer::boxed_owned(42, OwnedPointer::NULL));
        let shared_value2 = ValueContainer::Shared(SharedContainer::boxed_owned_mut(100, OwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_slot_count: 2,
            length: 1,
            injected_slots: vec![
                (0, ExternalSlotType::Shared(SharedSlotType::Ref)),
                (1, ExternalSlotType::Shared(SharedSlotType::RefMut)),
            ],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_remote_execution_block(exec_block_data, vec![Cow::Owned(shared_value1), Cow::Owned(shared_value2)]).unwrap();
        // should allocate slots and then compile the shared values into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::ALLOCATE_SLOT as u8,
                0, 0, 0, 0, // slot address of first value
                // compiled shared reference for first value
                InstructionCode::SHARED_REF as u8,
                1, // insert value
                0, 0, 0, 0, 0, // index of the first shared value
                InstructionCode::INT_32 as u8,
                42, 0, 0, 0, // value of the first shared integer
                InstructionCode::ALLOCATE_SLOT as u8,
                1, 0, 0, 0, // slot address of second value
                // compiled shared mutable reference for second value
                InstructionCode::SHARED_REF_MUT as u8,
                1, // insert value
                0, 0, 0, 0, 0, // index of the second shared value
                InstructionCode::INT_32 as u8,
                100, 0, 0, 0, // value of the second shared integer
                InstructionCode::NULL as u8, // body
            ]
        );
    }


    #[test]
    fn remote_execution_with_injected_moved_value() {
        let shared_value = ValueContainer::Shared(SharedContainer::boxed_owned(42, OwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_slot_count: 1,
            length: 1,
            injected_slots: vec![(0, ExternalSlotType::Shared(SharedSlotType::Move))],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_remote_execution_block(exec_block_data, vec![Cow::Owned(shared_value)]).unwrap();
        // should allocate slot and then compile the shared value into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::ALLOCATE_SLOT as u8,
                1, 0, 0, 0, // slot address of moved pointers
                // compiled shared moves
                InstructionCode::PERFORM_MOVE as u8,
                1, 0, 0, 0, // number of moves (1)
                0, 0, 0, 0, 0, // pointer address (assuming the shared container is stored at address 1)
                InstructionCode::ALLOCATE_SLOT as u8,
                0, 0, 0, 0, // slot address
                InstructionCode::GET_PROPERTY_INDEX as u8,
                0, 0, 0, 0, // index of the moved pointer
                InstructionCode::CLONE_SLOT as u8,
                1, 0, 0, 0, // slot address of the moved pointers
                InstructionCode::NULL as u8, // body
            ]
        );
    }

    #[test]
    fn remote_execution_moved_value_and_ref() {
        let shared_value1 = ValueContainer::Shared(SharedContainer::boxed_owned(42, OwnedPointer::NULL));
        let shared_value2 = ValueContainer::Shared(SharedContainer::boxed_owned_mut(100, OwnedPointer::NULL));
        let exec_block_data = InstructionBlockData {
            injected_slot_count: 2,
            length: 1,
            injected_slots: vec![
                (0, ExternalSlotType::Shared(SharedSlotType::Move)),
                (1, ExternalSlotType::Shared(SharedSlotType::Ref)),
            ],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_remote_execution_block(exec_block_data, vec![Cow::Owned(shared_value1), Cow::Owned(shared_value2)]).unwrap();
        // should allocate slots and then compile the shared values into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::ALLOCATE_SLOT as u8,
                2, 0, 0, 0, // slot address of moved pointers
                // compiled shared moves
                InstructionCode::PERFORM_MOVE as u8,
                1, 0, 0, 0, // number of moves (1)
                0, 0, 0, 0, 0, // pointer address (assuming the first shared container is stored at address 0)

                InstructionCode::ALLOCATE_SLOT as u8,
                0, 0, 0, 0, // slot address of first value (moved)
                InstructionCode::GET_PROPERTY_INDEX as u8,
                0, 0, 0, 0, // index of the moved pointer
                InstructionCode::CLONE_SLOT as u8,
                2, 0, 0, 0, // slot address of the moved pointers

                InstructionCode::ALLOCATE_SLOT as u8,
                1, 0, 0, 0, // slot address of second value
                // compiled shared reference for second value
                InstructionCode::SHARED_REF as u8,
                1, // insert value
                0, 0, 0, 0, 0, // index of the second shared value
                InstructionCode::INT_32 as u8,
                100, 0, 0, 0, // value of the second shared integer

                InstructionCode::NULL as u8, // body
            ]
        );
    }
}