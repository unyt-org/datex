use crate::core_compiler::value_compiler::{append_shared_container};
use crate::global::instruction_codes::InstructionCode;
use crate::global::protocol_structures::external_slot_type::{ExternalSlotType, SharedSlotType};
use crate::global::protocol_structures::instructions::InstructionBlockData;
use crate::runtime::execution::ExecutionError;
use crate::utils::buffers::append_u32;
use crate::values::value_container::ValueContainer;

/// Compiles a remote execution block into a bytecode buffer, with the given instruction block metadata and injected values
/// which can then be sent to another endpoint
pub fn compile_remote_execution_block(
    exec_block_data: InstructionBlockData,
    slot_values: &[&ValueContainer],
) -> Result<Vec<u8>, ExecutionError> {
    // build dxb
    let mut buffer = Vec::with_capacity(256);
    for (slot_addr, (_, external_slot_type)) in exec_block_data
        .injected_slots
        .into_iter()
        .enumerate()
    {
        buffer.push(
            InstructionCode::ALLOCATE_SLOT
                as u8,
        );
        append_u32(&mut buffer, slot_addr as u32);

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
                            SharedSlotType::Move => shared_container.clone().assert_owned()
                                .map_err(|_| ExecutionError::ExpectedOwnedSharedValue)?,
                            SharedSlotType::Ref => shared_container.derive_reference(),
                            SharedSlotType::RefMut => shared_container.try_derive_mutable_reference()
                                .map_err(|_| ExecutionError::MutableReferenceToNonMutableValue)?,
                        };
                        append_shared_container(
                            &mut buffer,
                            shared_container,
                            true
                        );
                    }
                }

            }
        }
    }
    buffer.extend_from_slice(
        &exec_block_data.body,
    );

    Ok(buffer)
}


#[cfg(test)]
mod tests {
    use crate::global::instruction_codes::InstructionCode;
    use crate::global::protocol_structures::external_slot_type::{ExternalSlotType, SharedSlotType};
    use crate::global::protocol_structures::instructions::InstructionBlockData;
    use crate::runtime::execution::execution_loop::remote_execution_blocks::compile_remote_execution_block;
    use crate::shared_values::pointer::{Pointer};
    use crate::shared_values::shared_container::SharedContainer;
    use crate::values::value_container::ValueContainer;

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
        let shared_value = ValueContainer::Shared(SharedContainer::boxed(42, Pointer::NULL));
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
                InstructionCode::SHARED_MOVE as u8, // shared ref instruction
                1, // value is inserted flag
                0, 0, 0, 0, 0, // pointer address (assuming the shared container is stored at address 1)
                InstructionCode::INT_32 as u8, // value type
                42, 0, 0, 0, // value data
                InstructionCode::NULL as u8, // body
            ]
        );
    }
}