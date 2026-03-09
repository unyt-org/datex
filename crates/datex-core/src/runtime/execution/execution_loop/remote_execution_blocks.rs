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
    slot_values: &[&ValueContainer],
) -> Result<Vec<u8>, ExecutionError> {
    let mut buffer = Vec::with_capacity(256);

    let moved_pointers_slot_index = exec_block_data
        .injected_slots
        .len() as u32;

    let moved_pointers = compile_preamble(&mut buffer, moved_pointers_slot_index, exec_block_data.clone(), slot_values)?;

    // if there are any moved pointers, we need to compile the preform_move instruction and allocate a slot for the moved pointers
    if moved_pointers.len() > 0 {
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
    slot_values: &[&ValueContainer],
) -> Result<Vec<SharedContainer>, ExecutionError> {

    let mut moved_pointers = vec![];

    // build dxb
    for (slot_addr, (_, external_slot_type)) in exec_block_data
        .injected_slots
        .into_iter()
        .enumerate()
    {
        buffer.push(
            InstructionCode::ALLOCATE_SLOT
                as u8,
        );
        append_u32(buffer, slot_addr as u32);

        let slot_value = &slot_values[slot_addr];

        match external_slot_type {
            ExternalSlotType::Local(_) => {
                todo!()
            },
            ExternalSlotType::Shared(shared_slot_type) => {
                match slot_value {
                    ValueContainer::Local(_) => {
                        return Err(ExecutionError::ExpectedSharedValue);
                    },
                    ValueContainer::Shared(shared_container) => {
                        let shared_container = match shared_slot_type {
                            SharedSlotType::Move => {
                                // get moved value from moved_pointers_slot
                                let index = moved_pointers.len() as u32;
                                shared_container.assert_owned().map_err(|_| ExecutionError::ExpectedOwnedSharedValue)?;
                                moved_pointers.push(shared_container.clone());
                                append_instruction_code(buffer, InstructionCode::GET_PROPERTY_INDEX.into());
                                append_u32(buffer, index);
                                append_instruction_code(buffer, InstructionCode::GET_SLOT.into());
                                append_u32(buffer, moved_pointers_slot_index);
                                continue;
                            },
                            SharedSlotType::Ref => shared_container.derive_reference(),
                            SharedSlotType::RefMut => shared_container.try_derive_mutable_reference()
                                .map_err(|_| ExecutionError::MutableReferenceToNonMutableValue)?,
                        };
                        append_shared_container(
                            buffer,
                            shared_container,
                            true
                        ).map_err(|_| ExecutionError::ExpectedOwnedSharedValue)?;
                    }
                }

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
    append_instruction_code(buffer, InstructionCode::ALLOCATE_SLOT.into());
    append_u32(buffer, moved_pointers_slot_index);

    append_perform_moves(
        buffer,
        moved_pointers
    ).unwrap(); // we already ensured that all moved pointers are owned local shared containers, so this should never fail
}


#[cfg(test)]
mod tests {
    use crate::global::instruction_codes::InstructionCode;
    use crate::global::protocol_structures::external_slot_type::{ExternalSlotType, SharedSlotType};
    use crate::global::protocol_structures::instructions::InstructionBlockData;
    use crate::runtime::execution::execution_loop::remote_execution_blocks::compile_remote_execution_block;
    use crate::shared_values::pointer::{OwnedPointer, Pointer};
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
        let res = compile_remote_execution_block(exec_block_data, &[]).unwrap();
        assert_eq!(res, vec![InstructionCode::NULL as u8]);
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
        let res = compile_remote_execution_block(exec_block_data, &[&shared_value]).unwrap();
        // should allocate slot and then compile the shared value into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::ALLOCATE_SLOT as u8,
                0, 0, 0, 0, // slot address
                // compiled shared moved container
                InstructionCode::PERFORM_MOVE as u8, // shared ref instruction
                1, // value is inserted flag
                0, 0, 0, 0, 0, // pointer address (assuming the shared container is stored at address 1)
                InstructionCode::INT_32 as u8, // value type
                42, 0, 0, 0, // value data
                InstructionCode::NULL as u8, // body
            ]
        );
    }
}