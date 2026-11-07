//! Compiler for update operations on shared containers into DXB instructions.

use crate::{
    core_compiler::{
        core_compilation_context::{
            CompileInput, CoreCompilationContext, DXBWithSharedValues,
        },
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    global::stack_index::StackIndex,
    instruction::{
        instruction_data::SharedRef, regular_instruction::RegularInstruction,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::{
        ReferenceMutability, SharedContainer, SharedContainerMutability,
    },
    value_updates::update_data::{
        AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
        ReplaceUpdateData, SetEntryUpdateData, UpdateData, UpdateOperation,
    },
    values::value_container::ValueContainer,
};

/// Compiles an update operation on a shared container into DXB
/// e.g. [UpdateData::SetEntry] to $x.a = b;
pub fn compile_updates(
    container: &SharedContainer,
    updates: &[&UpdateData],
    compile_input: CompileInput,
    cache: &mut SharedReferencesCache,
) -> DXBWithSharedValues {
    CoreCompilationContext::new(compile_input)
        .compile_to_dxb_with_shared_values(&(container, updates), cache)
}

impl ToInstructions for (&SharedContainer, &[&UpdateData]) {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        let (container, updates) = self;
        Box::new(gen move {
            yield_all!(append_updates(visitor, cache, container, updates));
        })
    }
}

/// Appends a list of update operations on a shared container
/// The updates are appended as a single statement block, with the shared container being pushed to the stack first, followed by each update operation.
gen fn append_updates<'a>(
    visitor: &mut dyn ValueVisitor,
    cache: &mut SharedReferencesCache,
    container: &SharedContainer,
    updates: &[&UpdateData],
) -> crate::instruction::Instruction {
    let statements_count = 1 + updates.len() as u32; // 1 for the push to stack, and 1 for each update operation

    yield RegularInstruction::statements(statements_count, true).into();
    yield RegularInstruction::push_to_stack().into();
    yield RegularInstruction::shared_ref(SharedRef {
        address: container.pointer_address(),
        ref_mutability: ReferenceMutability::Mutable, // can always be upgraded to mutable since the executing endpoint is the owner
        container_mutability: SharedContainerMutability::Mutable, // must be mutable for updates
    })
    .into();

    for update in updates {
        // TODO append update.path
        yield_all!(update.operation().to_instructions(visitor, cache));
    }
}

#[cfg(test)]
#[cfg(feature = "disassembler")]
mod tests {
    use crate::{
        core_compiler::{
            core_compilation_context::CompileInput,
            update_compiler::compile_updates,
        },
        disassembler::assertions::{assert_instructions_equal, instructions},
        global::stack_index::StackIndex,
        instruction::{
            instruction_data::{SharedRef, ShortTextData, UInt8Data},
            regular_instruction::RegularInstruction,
        },
        prelude::*,
        runtime::{
            cache::shared_references_cache::SharedReferencesCache,
            pointer_address_provider::SelfOwnedPointerAddressProvider,
            pointer_availability_lookup::PointerAvailabilityLookup,
        },
        shared_values::{
            ReferenceMutability, SharedContainer, SharedContainerMutability,
        },
        value_updates::update_data::{
            SetEntryUpdateData, UpdateData, UpdateOperation,
        },
        values::{core_values::map::Map, value_container::ValueContainer},
    };

    #[test]
    fn test_compile_update_set_entry() {
        let cache = &mut SharedReferencesCache::default();
        let container = SharedContainer::new_owned_with_inferred_allowed_type(
            Map::from(vec![(
                ValueContainer::from("test_key"),
                ValueContainer::from(42u8),
            )]),
            SharedContainerMutability::Mutable,
            &mut SelfOwnedPointerAddressProvider::default(),
        );

        let address = container.pointer_address();

        let update_data = UpdateData::new(UpdateOperation::set_entry(
            ValueContainer::from("test_key"),
            ValueContainer::from(100u8),
        ));

        let lookup = PointerAvailabilityLookup::default();

        let compile_input = CompileInput::new(&lookup, &[]);
        let dxb_with_shared_values =
            compile_updates(&container, &[&update_data], compile_input, cache);

        assert_instructions_equal!(
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
                    RegularInstruction::SetEntryText(ShortTextData(
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
        let cache = &mut SharedReferencesCache::default();
        let container = SharedContainer::new_owned_with_inferred_allowed_type(
            ValueContainer::from(42u8),
            SharedContainerMutability::Mutable,
            &mut SelfOwnedPointerAddressProvider::default(),
        );

        let address = container.pointer_address();

        let update_data = UpdateData::new(UpdateOperation::replace(
            ValueContainer::from(100u8),
        ));

        let lookup = PointerAvailabilityLookup::default();

        let compile_input = CompileInput::new(&lookup, &[]);
        let dxb_with_shared_values =
            compile_updates(&container, &[&update_data], compile_input, cache);

        assert_instructions_equal!(
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
