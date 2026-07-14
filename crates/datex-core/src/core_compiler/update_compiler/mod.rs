use crate::{
    core_compiler::{
        buffer_provider::BufferProvider,
        core_compilation_context::{
            CompileInput, CoreCompilationContext, DXBWithSharedValues,
        },
        value_compiler::append_regular_instruction,
        value_visitor::ValueVisitor,
    },
    global::protocol_structures::{
        instruction_data::{SharedRef, ShortTextData, StackIndex, UInt32Data},
        regular_instructions::RegularInstruction,
    },
    prelude::*,
    shared_values::{
        ReferenceMutability, SharedContainer, SharedContainerMutability,
    },
    value_updates::update_data::{
        AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
        ReplaceUpdateData, SetEntryUpdateData, UpdateData,
    },
    values::value_container::value_key::ValueKey,
};

/// Compiles an update operation on a shared container into DXB
/// e.g. [UpdateData::SetEntry] to $x.a = b;
pub fn compile_updates(
    container: &SharedContainer,
    updates: &[&UpdateData],
    compile_input: CompileInput,
) -> DXBWithSharedValues {
    let mut context =
        CoreCompilationContext::new(Vec::with_capacity(256), compile_input);
    append_updates(&mut context, container, updates);
    context.into_dxb_with_shared_values()
}

pub fn append_updates<T: BufferProvider + ValueVisitor>(
    context: &mut T,
    container: &SharedContainer,
    updates: &[&UpdateData],
) {
    let statements_count = 1 + updates.len() as u32; // 1 for the push to stack, and 1 for each update operation

    append_regular_instruction(
        context.cursor_mut(),
        RegularInstruction::statements(statements_count, true),
    );

    append_regular_instruction(
        context.cursor_mut(),
        RegularInstruction::PushToStack,
    );

    append_regular_instruction(
        context.cursor_mut(),
        RegularInstruction::SharedRef(SharedRef {
            address: container.pointer_address(),
            ref_mutability: ReferenceMutability::Mutable, // can always be upgraded to mutable since the executing endpoint is the owner
            container_mutability: SharedContainerMutability::Mutable, // must be mutable for updates
        }),
    );

    for update in updates {
        append_update(context, update);
    }
}

pub fn append_update<T: BufferProvider + ValueVisitor>(
    context: &mut T,
    update: &UpdateData,
) {
    match update {
        UpdateData::SetEntry(data) => append_set_entry(context, data),
        UpdateData::AppendEntry(data) => append_append_entry(context, data),
        UpdateData::Replace(data) => append_replace(context, data),
        UpdateData::Clear => append_clear(context),
        UpdateData::DeleteEntry(data) => append_delete_entry(context, data),
        UpdateData::ListSplice(data) => append_list_splice(context, data),
    }
}

pub fn append_set_entry<T: BufferProvider + ValueVisitor>(
    context: &mut T,
    set_entry_update_data: &SetEntryUpdateData,
) {
    // key
    append_set_property_value_key(context, set_entry_update_data.key.clone()); // TODO: ensure clone is ok here
    // value
    context.visit_value_container(set_entry_update_data.value.clone(), None); // TODO: ensure clone is ok here
    // target
    append_regular_instruction(
        context.cursor_mut(),
        RegularInstruction::BorrowStackValue(StackIndex(0)),
    );
}

pub fn append_replace<T: BufferProvider + ValueVisitor>(
    context: &mut T,
    replace_update_data: &ReplaceUpdateData,
) {
    append_regular_instruction(
        context.cursor_mut(),
        RegularInstruction::SetSharedContainerValue,
    );
    context.visit_value_container(replace_update_data.value.clone(), None); // TODO: ensure clone is ok here
    // target
    append_regular_instruction(
        context.cursor_mut(),
        RegularInstruction::BorrowStackValue(StackIndex(0)),
    );
}

pub fn append_append_entry<T: BufferProvider + ValueVisitor>(
    _context: &mut T,
    _append_entry_update_data: &AppendEntryUpdateData,
) {
    // +=
    todo!()
}

pub fn append_list_splice<T: BufferProvider + ValueVisitor>(
    _context: &mut T,
    _list_splice_update_data: &ListSpliceUpdateData,
) {
    todo!()
}
pub fn append_clear<T: BufferProvider + ValueVisitor>(_context: &mut T) {
    todo!()
}

pub fn append_delete_entry<T: BufferProvider + ValueVisitor>(
    _context: &mut T,
    _delete_entry_update_data: &DeleteEntryUpdateData,
) {
    todo!()
}

pub fn append_set_property_value_key<T: BufferProvider + ValueVisitor>(
    context: &mut T,
    value_key: ValueKey,
) {
    match value_key {
        ValueKey::Text(text) => append_regular_instruction(
            context.cursor_mut(),
            RegularInstruction::SetPropertyText(ShortTextData(text.clone())),
        ),
        ValueKey::Index(index) => append_regular_instruction(
            context.cursor_mut(),
            RegularInstruction::SetPropertyIndex(UInt32Data(index as u32)),
        ),
        ValueKey::Value(value) => {
            append_regular_instruction(
                context.cursor_mut(),
                RegularInstruction::SetPropertyDynamic,
            );
            context.visit_value_container(value, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core_compiler::core_compilation_context::CompileInput,
        disassembler::assertions::{
            assert_regular_instructions_equal, instructions,
        },
        global::protocol_structures::{
            instruction_data::UInt8Data,
            regular_instructions::RegularInstruction,
        },
        runtime::{
            pointer_address_provider::SelfOwnedPointerAddressProvider,
            pointer_availability_lookup::PointerAvailabilityLookup,
        },
        shared_values::{SharedContainer, SharedContainerMutability},
        value_updates::update_data::{SetEntryUpdateData, UpdateData},
        values::{
            core_values::map::Map,
            value_container::{ValueContainer, value_key::ValueKey},
        },
    };

    #[test]
    fn test_compile_update_set_entry() {
        let container = SharedContainer::new_owned_with_inferred_allowed_type(
            Map::from(vec![(
                ValueContainer::from("test_key".to_string()),
                ValueContainer::from(42u8),
            )]),
            SharedContainerMutability::Mutable,
            &mut SelfOwnedPointerAddressProvider::default(),
        );

        let address = container.pointer_address();

        let update_data = UpdateData::SetEntry(SetEntryUpdateData {
            key: ValueKey::Text("test_key".to_string()),
            value: ValueContainer::from(100u8),
        });

        let lookup = PointerAvailabilityLookup::default();

        let compile_input = CompileInput::new(&lookup, &[]);
        let dxb_with_shared_values =
            compile_updates(&container, &[&update_data], compile_input);

        assert_regular_instructions_equal!(
            &dxb_with_shared_values.dxb,
            (RegularInstruction::statements_with_children(
                true,
                instructions!(
                    RegularInstruction::PushToStack.with_children(
                        instructions!(RegularInstruction::SharedRef(
                            SharedRef {
                                address,
                                ref_mutability: ReferenceMutability::Mutable,
                                container_mutability:
                                    SharedContainerMutability::Mutable,
                            }
                        ),)
                    ),
                    RegularInstruction::SetPropertyText(ShortTextData(
                        "test_key".to_string()
                    ))
                    .with_children(instructions!(
                        RegularInstruction::UInt8(UInt8Data(100)),
                        RegularInstruction::BorrowStackValue(StackIndex(0)),
                    ))
                )
            ))
        )
    }

    #[test]
    fn test_compile_update_replace() {
        let container = SharedContainer::new_owned_with_inferred_allowed_type(
            ValueContainer::from(42u8),
            SharedContainerMutability::Mutable,
            &mut SelfOwnedPointerAddressProvider::default(),
        );

        let address = container.pointer_address();

        let update_data = UpdateData::Replace(ReplaceUpdateData {
            value: ValueContainer::from(100u8),
        });

        let lookup = PointerAvailabilityLookup::default();

        let compile_input = CompileInput::new(&lookup, &[]);
        let dxb_with_shared_values =
            compile_updates(&container, &[&update_data], compile_input);

        assert_regular_instructions_equal!(
            &dxb_with_shared_values.dxb,
            (RegularInstruction::statements_with_children(
                true,
                instructions!(
                    RegularInstruction::PushToStack.with_children(
                        instructions!(RegularInstruction::SharedRef(
                            SharedRef {
                                address,
                                ref_mutability: ReferenceMutability::Mutable,
                                container_mutability:
                                    SharedContainerMutability::Mutable,
                            }
                        ),)
                    ),
                    RegularInstruction::SetSharedContainerValue.with_children(
                        instructions!(
                            RegularInstruction::UInt8(UInt8Data(100)),
                            RegularInstruction::BorrowStackValue(StackIndex(0))
                        )
                    )
                )
            ))
        )
    }
}
