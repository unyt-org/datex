use crate::{
    core_compiler::{
        core_compilation_context::CoreCompilationContext,
        value_compiler::{
            SharedValueCompilationError, append_regular_instruction,
            append_value, append_value_container,
        },
    },
    global::protocol_structures::{
        injected_values::{
            InjectedValueDeclaration, InjectedValueType,
            SharedInjectedValueType,
        },
        instruction_data::{
            InstructionBlockData, PerformMove, RawSelfOwnedPointerAddress,
            SharedRef, SharedRefWithValue,
        },
        regular_instructions::RegularInstruction,
    },
    prelude::*,
    runtime::execution::ExecutionError,
    shared_values::{
        OwnedSharedContainer, PointerAddress, ReferencedSharedContainer,
        SharedContainer,
    },
    values::value_container::ValueContainer,
};

pub fn compile_injected_values(
    instruction_block_data: InstructionBlockData,
    injected_values: Vec<ValueContainer>,
) -> Result<(Vec<u8>, Vec<OwnedSharedContainer>), SharedValueCompilationError> {
    let mut context = CoreCompilationContext::new(Vec::new());
    compile_injected_values_with_context(
        &mut context,
        instruction_block_data.injected_values,
        injected_values,
    )?;
    let (mut buffer, values) = context.into_buffer_and_moved_values();
    // append instruction block body to buffer
    buffer.extend(instruction_block_data.body);
    Ok((buffer, values))
}

/// Prepends injected values to an instruction block
/// This is used for remote execution blocks and function bodies.
/// ```
/// #stack ..= (
///    #0 = MOVE (1,2,34);
///    -----
///    #parent = SHARED_REF 1;
///    #child = {p: #parent}
///    #parent.c = #child;
///    #3 = #0[1]
///    -----
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
/// )
/// ```
pub fn compile_injected_values_with_context(
    compilation_context: &mut CoreCompilationContext,
    injected_value_declarations: Vec<InjectedValueDeclaration>,
    injected_values: Vec<ValueContainer>,
) -> Result<(), SharedValueCompilationError> {
    if injected_value_declarations.len() != injected_values.len() {
        unreachable!(); // length must always match
    }

    for (injected_value_declaration, value_container) in
        injected_value_declarations
            .into_iter()
            .zip(injected_values.into_iter())
    {
        match injected_value_declaration.ty {
            InjectedValueType::Local(_ty) => match value_container {
                ValueContainer::Local(local_value) => {
                    append_value(compilation_context, local_value)?;
                }
                ValueContainer::Shared(_) => {
                    return Err(
                        SharedValueCompilationError::ExpectedLocalValue,
                    );
                }
            },
            InjectedValueType::Shared(ty) => match ty {
                SharedInjectedValueType::Move => match value_container {
                    ValueContainer::Shared(SharedContainer::Owned(
                        _owned_container,
                    )) => todo!(),
                    _ => return Err(
                        SharedValueCompilationError::ExpectedOwnedSharedValue,
                    ),
                },
                SharedInjectedValueType::Ref
                | SharedInjectedValueType::RefMut => match value_container {
                    ValueContainer::Shared(container) => {
                        let referenced_container = match container {
                            SharedContainer::Owned(owned_container) => {
                                owned_container.derive_with_max_mutability()
                            }
                            SharedContainer::Referenced(
                                referenced_container,
                            ) => referenced_container,
                        };
                        append_referenced_shared_container(
                            compilation_context,
                            referenced_container,
                            true,
                        )?;
                    }
                    _ => return Err(
                        SharedValueCompilationError::ExpectedOwnedSharedValue,
                    ),
                },
            },
        }
    }

    // compile preamble

    // ?
    Ok(())
}

fn append_referenced_shared_container(
    compilation_context: &mut CoreCompilationContext,
    referenced_container: ReferencedSharedContainer,
    insert_value: bool,
) -> Result<(), SharedValueCompilationError> {
    append_regular_instruction(
        compilation_context.cursor_mut(),
        RegularInstruction::PushToStack,
    );

    if insert_value {
        append_regular_instruction(
            compilation_context.cursor_mut(),
            RegularInstruction::SharedRefWithValue(SharedRefWithValue {
                address: match referenced_container.pointer_address() {
                    PointerAddress::SelfOwned(self_owned_address) => {
                        self_owned_address.into()
                    }
                    _ => unreachable!(), // referenced containers with insert_value=true should always be self owned
                },
                ref_mutability: referenced_container.reference_mutability(),
                container_mutability: referenced_container
                    .container_mutability(),
            }),
        );
        append_value_container(
            compilation_context,
            referenced_container.value_container().clone(),
        )?; // TODO: no clone
    } else {
        append_regular_instruction(
            compilation_context.cursor_mut(),
            RegularInstruction::SharedRef(SharedRef {
                address: referenced_container.pointer_address().clone().into(),
                ref_mutability: referenced_container.reference_mutability(),
                container_mutability: referenced_container
                    .container_mutability(),
            }),
        );
    }

    Ok(())
}

pub fn compile_shared_value_preamble(
    compilation_context: &mut CoreCompilationContext,
) {
    let shared_value_tracking = &compilation_context.shared_value_tracking;
    let cursor = &mut compilation_context.cursor;

    let moved_ptr_addresses =
        shared_value_tracking.get_moved_shared_addresses();

    append_regular_instruction(cursor, RegularInstruction::PushToStack);

    // push NULL to stack#1 if no moves
    if moved_ptr_addresses.is_empty() {
        append_regular_instruction(cursor, RegularInstruction::Null)
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
                            RawSelfOwnedPointerAddress {
                                bytes: shared_container.address,
                            },
                        )
                    })
                    .collect(),
            }),
        );
    }
}

fn compile_preamble(
    _moved_pointers_slot_index: u32,
    _exec_block_data: InstructionBlockData,
    _slot_values: Vec<ValueContainer>,
) -> Result<(Vec<u8>, Vec<OwnedSharedContainer>), ExecutionError> {
    let _context = CoreCompilationContext::new(Vec::new());
    todo!()
    // let mut moved_pointers: Vec<OwnedSharedContainer> = vec![];

    // // build dxb
    // for (
    //     slot_addr,
    //     (
    //         InjectedValueDeclaration {
    //             ty: external_slot_type,
    //             ..
    //         },
    //         slot_value,
    //     ),
    // ) in exec_block_data
    //     .injected_values
    //     .into_iter()
    //     .zip(slot_values.into_iter())
    //     .enumerate()
    // {
    //     context
    //         .cursor_mut()
    //         .write_all(&[InstructionCode::PUSH_TO_STACK as u8])
    //         .unwrap();
    //     append_u32(context.cursor_mut(), slot_addr as u32);
    //     match external_slot_type {
    //         InjectedValueType::Local(_) => {
    //             todo!()
    //         }
    //         InjectedValueType::Shared(shared_slot_type) => {
    //             let shared_container = match shared_slot_type {
    //                 SharedInjectedValueType::Move => {
    //                     // get moved value from moved_pointers_slot
    //                     let index = moved_pointers.len() as u32;

    //                     match slot_value {
    //                         BorrowedValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
    //                         BorrowedValueContainer::Shared(shared_container) => {
    //                             shared_container.assert_owned().map_err(|_| ExecutionError::ExpectedOwnedSharedValue)?;
    //                             moved_pointers.push(shared_container);
    //                             append_instruction_code_new(context.cursor_mut(), InstructionCode::TAKE_PROPERTY_INDEX);
    //                             append_u32(context.cursor_mut(), index);
    //                             append_instruction_code_new(context.cursor_mut(), InstructionCode::CLONE_STACK_VALUE);
    //                             append_u32(context.cursor_mut(), moved_pointers_slot_index);
    //                             continue;
    //                         }
    //                     }
    //                 },
    //                 SharedInjectedValueType::Ref => match slot_value {
    //                     BorrowedValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
    //                     BorrowedValueContainer::Shared(shared_container) => {
    //                         shared_container.derive_reference()
    //                     }
    //                 }
    //                 SharedInjectedValueType::RefMut => match slot_value {
    //                     BorrowedValueContainer::Local(_) => return Err(ExecutionError::ExpectedSharedValue),
    //                     BorrowedValueContainer::Shared(shared_container) => {
    //                         shared_container.try_derive_mutable_reference()
    //                             .map_err(|_| ExecutionError::MutableReferenceToNonMutableValue)?
    //                     }
    //                 }
    //             };

    //             append_shared_container(&mut context, &shared_container, true)
    //                 .unwrap();
    //         }
    //     }
    // }

    // Ok((context.into_buffer(), moved_pointers))
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_regular_instructions_equal,
        core_compiler::injected_values::compile_injected_values,
        global::{
            instruction_codes::InstructionCode,
            protocol_structures::{
                injected_values::{
                    InjectedValueDeclaration, InjectedValueType,
                    SharedInjectedValueType,
                },
                instruction_data::{
                    InstructionBlockData, Int32Data, PerformMove,
                    SharedRefWithValue, StackIndex, StatementsData, UInt32Data,
                },
                regular_instructions::RegularInstruction,
            },
        },
        prelude::*,
        runtime::{
            memory::Memory,
            pointer_address_provider::SelfOwnedPointerAddressProvider,
        },
        shared_values::{
            OwnedSharedContainer, PointerAddress, ReferenceMutability,
            SharedContainer, SharedContainerMutability,
        },
        values::value_container::ValueContainer,
    };

    #[test]
    fn remote_execution_no_injected_values() {
        let exec_block_data = InstructionBlockData {
            injected_value_count: 0,
            length: 1,
            injected_values: vec![],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_injected_values(exec_block_data, vec![]).unwrap().0;
        assert_regular_instructions_equal!(&res, [RegularInstruction::Null,])
    }

    #[test]
    fn remote_execution_with_injected_ref_value() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let memory = &Memory::default();

        let owned_container =
            OwnedSharedContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Immutable,
                address_provider,
            );
        let owned_address = owned_container.pointer_address().clone();

        let exec_block_data = InstructionBlockData {
            injected_value_count: 1,
            length: 1,
            injected_values: vec![InjectedValueDeclaration {
                index: StackIndex(0),
                ty: InjectedValueType::Shared(SharedInjectedValueType::Ref),
            }],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_injected_values(
            exec_block_data,
            vec![ValueContainer::Shared(SharedContainer::Owned(
                owned_container,
            ))],
        )
        .unwrap()
        .0;
        // should allocate slot and then compile the shared value into the buffer, followed by the body
        assert_regular_instructions_equal!(
            &res,
            [
                RegularInstruction::ShortStatements(StatementsData {
                    statements_count: 2,
                    terminated: false
                }),
                // ref
                RegularInstruction::PushToStack,
                RegularInstruction::SharedRefWithValue(SharedRefWithValue {
                    address: owned_address.into(),
                    ref_mutability: ReferenceMutability::Immutable,
                    container_mutability: SharedContainerMutability::Immutable
                }),
                RegularInstruction::Int32(Int32Data(42)),
                // original body
                RegularInstruction::Null,
            ]
        )
    }

    #[test]
    fn remote_execution_multiple_ref_values() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let shared_value1 =
            SharedContainer::new_owned_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Immutable,
                address_provider,
            );
        let shared_value2 =
            SharedContainer::new_owned_with_inferred_allowed_type(
                100,
                SharedContainerMutability::Mutable,
                address_provider,
            );
        let owned_address_1 = match shared_value1.pointer_address().clone() {
            PointerAddress::SelfOwned(address) => address,
            _ => unreachable!(),
        };
        let owned_address_2 = match shared_value2.pointer_address().clone() {
            PointerAddress::SelfOwned(address) => address,
            _ => unreachable!(),
        };

        let exec_block_data = InstructionBlockData {
            injected_value_count: 2,
            length: 1,
            injected_values: vec![
                InjectedValueDeclaration {
                    index: StackIndex(0),
                    ty: InjectedValueType::Shared(SharedInjectedValueType::Ref),
                },
                InjectedValueDeclaration {
                    index: StackIndex(1),
                    ty: InjectedValueType::Shared(
                        SharedInjectedValueType::RefMut,
                    ),
                },
            ],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_injected_values(
            exec_block_data,
            vec![
                ValueContainer::Shared(shared_value1),
                ValueContainer::Shared(shared_value2),
            ],
        )
        .unwrap()
        .0;
        // should allocate slots and then compile the shared values into the buffer, followed by the body
        assert_regular_instructions_equal!(
            &res,
            [
                RegularInstruction::ShortStatements(StatementsData {
                    statements_count: 3,
                    terminated: false
                }),
                // first ref
                RegularInstruction::PushToStack,
                RegularInstruction::SharedRefWithValue(SharedRefWithValue {
                    address: owned_address_1.into(),
                    ref_mutability: ReferenceMutability::Immutable,
                    container_mutability: SharedContainerMutability::Immutable
                }),
                RegularInstruction::Int32(Int32Data(42)),
                // second ref
                RegularInstruction::PushToStack,
                RegularInstruction::SharedRefWithValue(SharedRefWithValue {
                    address: owned_address_2.into(),
                    ref_mutability: ReferenceMutability::Mutable,
                    container_mutability: SharedContainerMutability::Mutable
                }),
                RegularInstruction::Int32(Int32Data(100)),
                // original body
                RegularInstruction::Null,
            ]
        );
    }

    #[test]
    fn remote_execution_with_injected_moved_value() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let shared_value =
            SharedContainer::new_owned_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Immutable,
                address_provider,
            );

        let owned_address = match shared_value.pointer_address().clone() {
            PointerAddress::SelfOwned(address) => address,
            _ => unreachable!(),
        };

        let exec_block_data = InstructionBlockData {
            injected_value_count: 1,
            length: 1,
            injected_values: vec![InjectedValueDeclaration {
                index: StackIndex(0),
                ty: InjectedValueType::Shared(SharedInjectedValueType::Move),
            }],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_injected_values(
            exec_block_data,
            vec![ValueContainer::Shared(shared_value)],
        )
        .unwrap()
        .0;
        // should allocate slot and then compile the shared value into the buffer, followed by the body
        assert_regular_instructions_equal!(
            &res,
            [
                RegularInstruction::ShortStatements(StatementsData {
                    statements_count: 3,
                    terminated: false
                }),
                // move
                RegularInstruction::PushToStack,
                RegularInstruction::PerformMove(PerformMove {
                    pointer_count: 1,
                    pointers: vec![(0, owned_address.into())]
                }),
                // get first moved value
                RegularInstruction::PushToStack,
                RegularInstruction::TakePropertyIndex(UInt32Data(0)),
                RegularInstruction::BorrowStackValue(StackIndex(0)),
                // original body
                RegularInstruction::Null,
            ]
        );
    }

    #[test]
    fn remote_execution_moved_value_and_ref() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let shared_value1 =
            SharedContainer::new_owned_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Immutable,
                address_provider,
            );
        let shared_value2 =
            SharedContainer::new_owned_with_inferred_allowed_type(
                100,
                SharedContainerMutability::Immutable,
                address_provider,
            );
        let exec_block_data = InstructionBlockData {
            injected_value_count: 2,
            length: 1,
            injected_values: vec![
                InjectedValueDeclaration {
                    index: StackIndex(0),
                    ty: InjectedValueType::Shared(
                        SharedInjectedValueType::Move,
                    ),
                },
                InjectedValueDeclaration {
                    index: StackIndex(1),
                    ty: InjectedValueType::Shared(SharedInjectedValueType::Ref),
                },
            ],
            body: vec![InstructionCode::NULL as u8],
        };
        let res = compile_injected_values(
            exec_block_data,
            vec![
                ValueContainer::Shared(shared_value1),
                ValueContainer::Shared(shared_value2),
            ],
        )
        .unwrap()
        .0;
        // should allocate slots and then compile the shared values into the buffer, followed by the body
        assert_eq!(
            res,
            vec![
                InstructionCode::SHORT_STATEMENTS as u8,
                4,
                0,
                InstructionCode::PUSH_TO_STACK as u8,
                2,
                0,
                0,
                0, // slot address of moved pointers
                // compiled shared moves
                InstructionCode::PERFORM_MOVE as u8,
                1,
                0,
                0,
                0, // number of moves (1)
                0, // immmut
                0,
                0,
                0,
                0,
                0, // pointer address (assuming the first shared container is stored at address 0)
                InstructionCode::PUSH_TO_STACK as u8,
                0,
                0,
                0,
                0, // slot address of first value (moved)
                InstructionCode::TAKE_PROPERTY_INDEX as u8,
                0,
                0,
                0,
                0, // index of the moved pointer
                InstructionCode::CLONE_STACK_VALUE as u8,
                2,
                0,
                0,
                0, // slot address of the moved pointers
                InstructionCode::PUSH_TO_STACK as u8,
                1,
                0,
                0,
                0, // slot address of second value
                // compiled shared reference for second value
                InstructionCode::SHARED_REF_WITH_VALUE as u8,
                0,
                0,
                0,
                0,
                0, // address of the second shared value
                0, // immutable ref
                1, // mutable value
                InstructionCode::INT_32 as u8,
                100,
                0,
                0,
                0, // value of the second shared integer
                InstructionCode::NULL as u8, // body
            ]
        );
    }
}
