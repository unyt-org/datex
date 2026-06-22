use crate::{
    collections::HashMap,
    core_compiler::{
        buffer_provider::BufferProvider,
        core_compilation_context::{ByteCursor, CoreCompilationContext},
        shared_value_tracking::TrackedValue,
        value_compiler::append_regular_instruction,
    },
    global::protocol_structures::{
        instruction_data::StackIndex, regular_instructions::RegularInstruction,
    },
    shared_values::SharedContainer,
    values::value_container::value_key::ValueKey,
};

#[derive(Debug)]
struct InsertedValueInfo {
    inner_stack_index: StackIndex,
    modification: Vec<ShellModifications>,
}

#[derive(Debug)]
struct PreambleContext<'a> {
    cursor: &'a mut ByteCursor,
    inserted_values: HashMap<SharedContainer, InsertedValueInfo>,
    current_stack_index: StackIndex,
}

impl BufferProvider for PreambleContext<'_> {
    fn cursor_mut(&mut self) -> &mut ByteCursor {
        self.cursor
    }
}

impl<'a> PreambleContext<'a> {
    pub fn try_get_stack_index_for_value(
        &self,
        value: &SharedContainer,
    ) -> Option<StackIndex> {
        self.inserted_values
            .get(value)
            .map(|info| info.inner_stack_index)
    }

    fn get_next_stack_index(&mut self) -> StackIndex {
        let stack_index = self.current_stack_index;
        self.current_stack_index = StackIndex(self.current_stack_index.0 + 1);
        stack_index
    }
}

#[derive(Debug)]
struct ShellModifications {
    target_index: StackIndex,
    assigned_property: ValueKey,
}

pub(super) fn append_injected_values_preamble(
    byte_cursor: &mut ByteCursor,
    tracked_values: Vec<TrackedValue>, // top level shared container roots
                                       // TODO: injected context to check if shared value must be inserted or is already known on receiver endpoint
) -> Vec<SharedContainer> {
    // no injected values
    if tracked_values.is_empty() {
        return Vec::new();
    }

    let context = &mut PreambleContext {
        cursor: byte_cursor,
        inserted_values: HashMap::new(),
        current_stack_index: StackIndex(0),
    };

    for container in tracked_values {
        append_injected_value(context, container);
    }
    todo!()

    // /**
    //  *
    //  * a = {}
    //  * b = {}
    //  * c = {}
    //  * a.b = b
    //  * b.c = c
    //  * c.a = a
    //  * c.b = b
    //  *
    //  * a = {b: b}
    //  * b = {c: c}
    //  * c = {b: b, a: a}
    //  *
    //  *
    //  *
    //  * c = {b: null, a: null} -> (c.b = ?b, c.a = ?a)
    //  * a = {b: null} -> (a.b = ?b, ?b = ?c.b)
    //  * b = {a: null}
    //  *
    //  *
    //  *
    //  *
    //  *
    // #0 = 42;
    // #1 = shared 43;
    // #2 = null
    // #2.x = shared [34,43 ,344334#0, #2];
    // #4 = {}; <---  #3 -> #4.(#5) = #3
    // #3 = {a: #1, b: #4}
    // #3.b = #4;
    // **/
}

fn append_injected_value(
    context: &mut PreambleContext,
    shared_container: TrackedValue,
) {
    if let Some(index) =
        context.try_get_stack_index_for_value(shared_container.container())
    {
        append_regular_instruction(
            context.cursor,
            RegularInstruction::BorrowStackValue(index),
        );
    } else {
    }
    todo!()
    // if context.inserted_values.contains_key(shared_container) {
    //     // #0 = 'shared x
    //     // #1 = {a: 'mut shared}
    //     append_regular_instruction(
    //         context.byte_cursor,
    //         RegularInstruction::BorrowStackValue(
    //             context.inserted_values[shared_container],
    //         ),
    //     );
    // }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_regular_instructions_equal,
        core_compiler::{
            core_compilation_context::ByteCursor,
            preamble::append_injected_values_preamble,
            shared_value_tracking::TrackedValue,
            value_compiler::append_regular_instruction,
        },
        global::protocol_structures::{
            instruction_data::{
                Int32Data, ListData, SharedRefWithValue, StackIndex,
            },
            regular_instructions::RegularInstruction,
        },
        runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
        shared_values::{
            PointerAddress, ReferenceMutability, SelfOwnedPointerAddress,
            SelfOwnedSharedContainer, SharedContainer,
            SharedContainerMutability, SharedContainerOwnership,
        },
        values::value_container::ValueContainer,
    };

    fn assert_preamble_instructions(
        shared_containers: Vec<TrackedValue>,
        instructions: Vec<RegularInstruction>,
    ) {
        let mut cursor = ByteCursor::new(vec![]);

        // mock body
        append_regular_instruction(&mut cursor, RegularInstruction::Null);

        append_injected_values_preamble(&mut cursor, shared_containers);

        assert_regular_instructions_equal!(&cursor.into_inner(), instructions);
    }

    fn generate_shared_value_from_value_container(
        address_provider: &mut SelfOwnedPointerAddressProvider,
        value: impl Into<ValueContainer>,
        ownership: SharedContainerOwnership,
        mutability: SharedContainerMutability,
    ) -> (SharedContainer, SelfOwnedPointerAddress) {
        let value_container = value.into();
        let mut shared_container =
            SharedContainer::new_owned_with_inferred_allowed_type(
                value_container,
                mutability,
                address_provider,
            );
        let address = match &shared_container {
            SharedContainer::Owned(owned_container) => {
                owned_container.pointer_address().clone()
            }
            _ => unreachable!(),
        };

        match ownership {
            SharedContainerOwnership::Referenced(mutability) => {
                shared_container = SharedContainer::Referenced(
                    shared_container
                        .try_derive_reference_with_mutability(mutability)
                        .unwrap(),
                );
            }
            _ => {}
        };

        (shared_container, address)
    }

    #[test]
    fn preamble_no_injected_values() {
        assert_preamble_instructions(vec![], vec![RegularInstruction::Null]);
    }

    #[test]
    fn preamble_single_non_referencing_ref() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let (reference, address) = generate_shared_value_from_value_container(
            address_provider,
            42,
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Immutable,
            ),
            SharedContainerMutability::Immutable,
        );

        assert_preamble_instructions(
            vec![TrackedValue::Root {
                container: reference,
                index: StackIndex(1),
            }],
            vec![
                RegularInstruction::statements(2, false),
                // preamble
                RegularInstruction::statements(2, false),
                // ref
                RegularInstruction::PushToStack,
                RegularInstruction::SharedRefWithValue(SharedRefWithValue {
                    address: address.into(),
                    ref_mutability: ReferenceMutability::Immutable,
                    container_mutability: SharedContainerMutability::Immutable,
                }),
                RegularInstruction::Int32(Int32Data(42)),
                RegularInstruction::ShortList(ListData { element_count: 1 }),
                RegularInstruction::TakeStackValue(StackIndex(0)),
                // body
                RegularInstruction::Null,
            ],
        );
    }
}
