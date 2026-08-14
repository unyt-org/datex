use crate::{
    collections::HashMap,
    core_compiler::{
        buffer_provider::BufferProvider,
        core_compilation_context::ByteCursor,
        shared_value_tracking::{TrackedValueCollection, TrackedValueMetadata},
        to_instructions::ToInstructions,
        type_compiler::append_type_instruction,
        update_compiler::append_set_property_value_key,
        value_compiler::append_value,
        value_visitor::{ParentAccessor, ParentContext, ValueVisitor},
    },
    global::protocol_structures::{
        instruction_data::{
            MoveWithValue, SharedRef, SharedRefWithValue, StackIndex,
        },
        regular_instructions::RegularInstruction,
    },
    prelude::*,
    shared_values::{
        OwnedSharedContainer, PointerAddress, ReferencedSharedContainer,
        SharedContainer, SharedContainerMutability,
        traits::SharedContainerCommon,
    },
    types::r#type::Type,
    values::value_container::{ValueContainer, value_key::ValueKey},
};
use binrw::{BinWrite, io::Write};

#[derive(Debug)]
enum VisitedValue {
    /// Indicates that the shared container was already inserted and is available at the given stack index
    Inserted { stack_index: StackIndex },
    /// Indicates that the shared container is referenced by one or multiple shared containers that were already inserted
    Required { parent_contexts: Vec<ParentContext> },
}

#[derive(Debug)]
struct PreambleContext<'a> {
    cursor: &'a mut ByteCursor,
    visited_values: HashMap<SharedContainer, VisitedValue>,
    /// Maps original parent stack indices to a new stack index for a reference that should be used
    reference_indices: HashMap<StackIndex, StackIndex>,
    current_stack_index: StackIndex,
}

impl PreambleContext<'_> {
    /// Appends a borrow instruction for the given stack index, using the reference index if it exists
    fn append_borrow_for_index(&mut self, index: StackIndex) {
        let index = self.resolve_index(index);
        RegularInstruction::borrow_stack_value(index)
            .write(self.cursor)
            .expect("Failed to write borrow instruction");
    }

    fn resolve_index(&self, index: StackIndex) -> StackIndex {
        self.reference_indices.get(&index).cloned().unwrap_or(index)
    }
}

impl BufferProvider for PreambleContext<'_> {
    fn cursor_mut(&mut self) -> &mut ByteCursor {
        self.cursor
    }
}
// var x = shared {a: {b: null}};
// x.a.b = x; /// #0.a.b = x;

impl ValueVisitor for PreambleContext<'_> {
    fn visit_value_container(
        &mut self,
        value_container: ValueContainer,
        parent_context: Option<ParentContext>,
    ) {
        match value_container {
            ValueContainer::Local(value) => {
                append_value(self, value, parent_context)
            }
            ValueContainer::Shared(shared_container) => {
                let parent_context = parent_context.expect("no parent context");
                let key = tracking_key(&shared_container);

                match self.visited_values.get_mut(&key) {
                    // shared container was not yet inserted, keep track as required dependency
                    None => {
                        self.visited_values.insert(
                            key,
                            VisitedValue::Required {
                                parent_contexts: vec![parent_context],
                            },
                        );
                        // placeholder, will be later replaced via setter
                        self.write(RegularInstruction::null());
                    }
                    Some(VisitedValue::Required {
                        parent_contexts: partial_instantiations,
                    }) => {
                        partial_instantiations.push(parent_context);
                        // placeholder, will be later replaced via setter
                        self.write(RegularInstruction::null());
                    }
                    // shared container has already been inserted, use stack value
                    // depending on the move flag, we either take the value or borrow it
                    Some(VisitedValue::Inserted { stack_index }) => {
                        if shared_container.treat_as_move() {
                            RegularInstruction::take_stack_value(*stack_index)
                                .write(self.cursor)
                                .unwrap();
                        } else {
                            let index = self
                                .reference_indices
                                .get(stack_index)
                                .cloned()
                                .unwrap_or(*stack_index);
                            RegularInstruction::borrow_stack_value(index)
                                .write(self.cursor)
                                .unwrap();
                        }
                    }
                }
            }
        }
    }

    fn visit_type(&mut self, ty: Type) {
        match ty {
            // intercept shared types
            Type::Entity(_) => todo!(),
            _ => {
                let instructions = ty.to_instructions(None).collect::<Vec<_>>();
                for instruction in instructions {
                    append_type_instruction(self.cursor_mut(), instruction);
                }
            }
        }
    }
}

impl<'a> PreambleContext<'a> {
    fn get_next_stack_index(&mut self) -> StackIndex {
        let stack_index = self.current_stack_index;
        self.current_stack_index = StackIndex(self.current_stack_index.0 + 1);
        stack_index
    }
}

pub(super) fn append_injected_values_preamble(
    collection: TrackedValueCollection, // top level shared container roots
    body: Vec<u8>,
) -> (Vec<u8>, Vec<SharedContainer>) {
    // no injected values
    if !collection.has_tracked_values() {
        return (body, vec![]);
    }

    let tracked_values = collection.tracked_values;
    let mut inner_cursor = ByteCursor::new(vec![]);

    let mut root_containers =
        Vec::<SharedContainer>::with_capacity(collection.root_count);
    let mut root_container_stack_indices: Vec<Option<StackIndex>> =
        vec![None; collection.root_count];

    // This keeps track of the number of inner statements of the preamble.
    // The count is the number of all stack values, plus the number of patches per stack value applied, plus one for the list return
    let inner_statement_count = &mut 0u32;

    {
        let context = &mut PreambleContext {
            cursor: &mut inner_cursor,
            visited_values: HashMap::new(),
            reference_indices: HashMap::new(),
            current_stack_index: StackIndex(0),
        };

        // loop through the tracked values in reverse order (as optimization)
        for (tracked_value, metadata) in tracked_values.into_iter().rev() {
            let index = append_injected_value(
                context,
                &tracked_value,
                &metadata,
                inner_statement_count,
            );

            if let TrackedValueMetadata::Root {
                index: root_stack_index,
                ..
            } = metadata
            {
                // if it is a root tracked value, register in the root container list
                root_container_stack_indices[root_stack_index.0 as usize] =
                    Some(index);
                root_containers.push(tracked_value);
            }
        }

        // asserting that root stack index keys are a contiguous list of 0-n -> inner stack indices
        let root_container_stack_indices_sorted = root_container_stack_indices
            .iter()
            .map(|index| {
                index.expect("Root stack index should have been registered")
            })
            .collect::<Vec<_>>();

        // Final root list
        context.write(RegularInstruction::list(
            root_container_stack_indices_sorted.len() as u32,
        ));

        // insert all root stack values into the list, which is the final value of the preamble
        // append [#0,...#x]
        for stack_index in root_container_stack_indices_sorted {
            context.write(RegularInstruction::take_stack_value(stack_index));
        }

        // The final root list is also one inner statement.
        *inner_statement_count += 1;
    }
    let mut byte_cursor = ByteCursor::new(vec![]);

    // statements (preamble + body) start
    RegularInstruction::statements(2, false)
        .write(&mut byte_cursor)
        .unwrap();

    // the preamble pushes a list of the root shared containers to the stack
    RegularInstruction::push_list_to_stack()
        .write(&mut byte_cursor)
        .unwrap();

    // the inner preamble statements, which push the shared containers to the stack
    // and apply the patches
    RegularInstruction::statements(*inner_statement_count, false)
        .write(&mut byte_cursor)
        .unwrap();
    // the inner preamble content, which pushes the shared containers to the stack and applies the patches
    byte_cursor.write_all(&inner_cursor.into_inner()).unwrap();

    // append body
    byte_cursor.write_all(&body).unwrap();
    (byte_cursor.into_inner(), root_containers)
}

fn tracking_key(container: &SharedContainer) -> SharedContainer {
    SharedContainer::Referenced(
        container.derive_reference_with_max_mutability(), // TODO: why no clone? (hash match)
    )
}

fn append_injected_value(
    context: &mut PreambleContext,
    container: &SharedContainer,
    metadata: &TrackedValueMetadata,
    statements_count: &mut u32,
) -> StackIndex /* the patches count to calc statement count */ {
    let key = tracking_key(container);

    let index =
        push_injected_container(context, container, metadata, statements_count);

    let pending_contexts = match context.visited_values.remove(&key) {
        Some(VisitedValue::Required { parent_contexts }) => parent_contexts,
        // Some(VisitedValue::Inserted { .. }) => {
        //     unreachable!("Container was marked inserted while still compiling")
        // }
        _ => vec![],
    };

    context
        .visited_values
        .insert(key, VisitedValue::Inserted { stack_index: index });

    let patch_count = pending_contexts.len() as u32;
    *statements_count += patch_count;

    for parent_context in pending_contexts {
        let ParentContext {
            parent,
            mut accessors,
        } = parent_context;
        let parent_key = tracking_key(&parent);

        let assigned_property = accessors.pop().expect("FIXME");

        let parent_stack_index = match context.visited_values.get(&parent_key) {
            Some(VisitedValue::Inserted { stack_index }) => *stack_index,
            Some(VisitedValue::Required { .. }) => {
                unreachable!("Deferred assignment parent is not inserted yet");
            }
            None => {
                unreachable!("Deferred assignment parent was not registered");
            }
        };
        match assigned_property {
            ParentAccessor::ValueKey(key) => {
                append_set_property_value_key(context, key);
            }
            _ => todo!(),
        }

        // first target, or first instru?
        context.append_borrow_for_index(index);
        append_property_target(context, parent_stack_index, accessors);
    }

    index
}

fn append_property_target(
    context: &mut PreambleContext,
    parent_stack_index: StackIndex,
    mut accessors: Vec<ParentAccessor>,
) {
    // this is the last accessor, we can now set the property on the parent stack value
    // if there are more accessors, we will recurse below
    let Some(accessor) = accessors.pop() else {
        context.append_borrow_for_index(parent_stack_index);
        return;
    };

    match accessor {
        ParentAccessor::ValueKey(key) => match key {
            ValueKey::Index(property) => {
                context.write(RegularInstruction::get_entry_index(
                    property as u32,
                ));
            }
            ValueKey::Text(property) => {
                context.write(RegularInstruction::get_entry_text(property));
            }
            ValueKey::Value(value) => {
                context.write(RegularInstruction::get_entry_dynamic());
                context.visit_value_container(value, None);
            }
        },
        _ => {
            todo!("Dynamic deferred property access");
        }
    }

    append_property_target(context, parent_stack_index, accessors);
}

fn push_injected_container(
    context: &mut PreambleContext,
    container: &SharedContainer,
    metadata: &TrackedValueMetadata,
    statements_count: &mut u32,
) -> StackIndex {
    *statements_count += 1;

    let index = context.get_next_stack_index();

    // append push to stack
    context.write(RegularInstruction::push_to_stack());

    match container {
        SharedContainer::Owned(owned_container) => {
            append_move_with_value(context, owned_container);

            // add additional reference so that container can later still be accessed
            // TODO: optimize this, only include if actually needed later
            append_additional_reference(context, owned_container, index);
            *statements_count += 1;
        }
        SharedContainer::Referenced(referenced_container) => {
            if !metadata.is_known() {
                append_referenced_shared_container_with_value(
                    context,
                    referenced_container,
                );
            } else {
                append_referenced_shared_container(
                    context,
                    referenced_container,
                );
            }
        }
    }

    index
}

fn append_additional_reference(
    context: &mut PreambleContext,
    owned_container: &OwnedSharedContainer,
    original_index: StackIndex,
) {
    // append push to stack
    context.write(RegularInstruction::push_to_stack());
    // append push to stack
    context.write(match owned_container.container_mutability() {
        SharedContainerMutability::Mutable => {
            RegularInstruction::get_stack_value_shared_ref_mut(original_index)
        }
        SharedContainerMutability::Immutable => {
            RegularInstruction::get_stack_value_shared_ref(original_index)
        }
    });

    let reference_index = context.get_next_stack_index();
    context
        .reference_indices
        .insert(original_index, reference_index);
}

/// Appends a move instruction to the preamble to move an owned shared containers
fn append_move_with_value(
    context: &mut PreambleContext,
    owned_container: &OwnedSharedContainer,
) {
    context.write(RegularInstruction::move_with_value(MoveWithValue {
        mutability: owned_container.container_mutability(),
        previous_address: owned_container.pointer_address().clone(),
    }));

    let inner = owned_container.value_container().clone();
    match inner {
        ValueContainer::Local(value) => append_value(
            context,
            value,
            Some(ParentContext::new(SharedContainer::Referenced(
                owned_container.derive_with_max_mutability(),
            ))),
        ),
        _container @ ValueContainer::Shared(_) => {
            context.visit_value_container(
                owned_container.value_container().clone(),
                Some(ParentContext::new(SharedContainer::Referenced(
                    owned_container.derive_with_max_mutability(),
                ))),
            );
        }
    }
}

fn append_referenced_shared_container(
    context: &mut PreambleContext,
    referenced_container: &ReferencedSharedContainer,
) {
    match referenced_container.pointer_address() {
        PointerAddress::SelfOwned(pointer_address) => {
            context.write(RegularInstruction::shared_ref(SharedRef {
                address: PointerAddress::SelfOwned(pointer_address),
                ref_mutability: referenced_container.reference_mutability(),
                container_mutability: referenced_container
                    .container_mutability(),
            }));
        }
        PointerAddress::Remote(pointer_address) => {
            context.write(RegularInstruction::shared_ref(SharedRef {
                address: PointerAddress::Remote(pointer_address),
                ref_mutability: referenced_container.reference_mutability(),
                container_mutability: referenced_container
                    .container_mutability(),
            }));
        }
    }
}

fn append_referenced_shared_container_with_value(
    context: &mut PreambleContext,
    referenced_container: &ReferencedSharedContainer,
) {
    match referenced_container.pointer_address() {
        // insert with value for self owned references
        PointerAddress::SelfOwned(pointer_address) => {
            context.write(RegularInstruction::shared_ref_with_value(
                SharedRefWithValue {
                    address: pointer_address,
                    ref_mutability: referenced_container.reference_mutability(),
                    container_mutability: referenced_container
                        .container_mutability(),
                },
            ));
            // TODO: no clone?
            context.visit_value_container(
                referenced_container.value_container().clone(),
                Some(ParentContext::new(SharedContainer::Referenced(
                    referenced_container.clone(),
                ))),
            );
        }
        // insert without value for non self owned references
        PointerAddress::Remote(pointer_address) => {
            context.write(RegularInstruction::shared_ref(SharedRef {
                address: PointerAddress::Remote(pointer_address),
                ref_mutability: referenced_container.reference_mutability(),
                container_mutability: referenced_container
                    .container_mutability(),
            }));
        }
    }
}

#[cfg(test)]
#[cfg(feature = "disassembler")]
mod tests {
    use binrw::BinWrite;

    use crate::{
        core_compiler::{
            core_compilation_context::ByteCursor,
            preamble::append_injected_values_preamble,
            shared_value_tracking::{
                TrackedValueCollection, TrackedValueMetadata,
            },
        },
        disassembler::{
            InstructionTree,
            assertions::{assert_instructions_equal, instructions},
            print_disassembled,
        },
        global::protocol_structures::{
            instruction_data::{
                Int32Data, ListData, MoveWithValue, SharedRefWithValue,
                ShortListData, ShortMapData, ShortTextData, StackIndex,
                UInt32Data,
            },
            instructions::Instruction,
            regular_instructions::RegularInstruction,
        },
        prelude::*,
        runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
        shared_values::{
            OwnedSharedContainer, PointerAddress, ReferenceMutability,
            ReferencedSharedContainer, SelfOwnedPointerAddress,
            SelfOwnedSharedContainer, SharedContainer,
            SharedContainerMutability, SharedContainerOwnership,
            traits::SharedContainerCommon,
        },
        values::{
            core_values::{list::List, map::Map},
            value_container::ValueContainer,
        },
    };

    fn assert_preamble_instructions(
        tracked_values: Vec<(SharedContainer, TrackedValueMetadata)>,
        instructions: Vec<impl Into<InstructionTree<Instruction>>>,
    ) {
        // mock body
        let mut cursor = ByteCursor::new(vec![]);
        RegularInstruction::null().write(&mut cursor).unwrap();

        let root_count = tracked_values
            .iter()
            .filter(|(_, metadata)| {
                matches!(metadata, TrackedValueMetadata::Root { .. })
            })
            .count();

        let collection = TrackedValueCollection {
            root_count,
            tracked_values,
        };
        let (bytes, _) =
            append_injected_values_preamble(collection, cursor.into_inner());

        assert_instructions_equal!(&bytes, instructions);
    }

    fn generate_shared_owned_value(
        address_provider: &mut SelfOwnedPointerAddressProvider,
        value: impl Into<ValueContainer>,
        mutability: SharedContainerMutability,
    ) -> (OwnedSharedContainer, SelfOwnedPointerAddress) {
        match generate_shared_value(
            address_provider,
            value,
            SharedContainerOwnership::Owned,
            mutability,
        ) {
            (SharedContainer::Owned(owned_container), address) => {
                (owned_container, address)
            }
            _ => unreachable!(),
        }
    }

    fn generate_shared_referenced_value(
        address_provider: &mut SelfOwnedPointerAddressProvider,
        value: impl Into<ValueContainer>,
        reference_mutability: ReferenceMutability,
        mutability: SharedContainerMutability,
    ) -> (ReferencedSharedContainer, SelfOwnedPointerAddress) {
        match generate_shared_value(
            address_provider,
            value,
            SharedContainerOwnership::Referenced(reference_mutability),
            mutability,
        ) {
            (SharedContainer::Referenced(referenced_container), address) => {
                (referenced_container, address)
            }
            _ => unreachable!(),
        }
    }

    fn generate_shared_value(
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

        if let SharedContainerOwnership::Referenced(mutability) = ownership {
            shared_container = SharedContainer::Referenced(
                shared_container
                    .try_derive_reference_with_mutability(mutability)
                    .unwrap(),
            );
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
        let (reference, address) = generate_shared_referenced_value(
            address_provider,
            42,
            ReferenceMutability::Immutable,
            SharedContainerMutability::Immutable,
        );

        assert_preamble_instructions(
            vec![(
                SharedContainer::Referenced(reference),
                TrackedValueMetadata::Root {
                    index: StackIndex(0),
                    is_known: false,
                },
            )],
            vec![RegularInstruction::statements_with_children(
                false,
                instructions!(
                    // preamble
                    RegularInstruction::PushListToStack,
                    RegularInstruction::statements_with_children(
                        false,
                        instructions!(
                            RegularInstruction::PushToStack,
                            RegularInstruction::SharedRefWithValue(
                                SharedRefWithValue {
                                    address,
                                    ref_mutability:
                                        ReferenceMutability::Immutable,
                                    container_mutability:
                                        SharedContainerMutability::Immutable,
                                }
                            ),
                            RegularInstruction::Int32(Int32Data(42)),
                            RegularInstruction::list_with_children(
                                instructions!(
                                    RegularInstruction::TakeStackValue(
                                        StackIndex(0)
                                    )
                                )
                            ),
                        )
                    ),
                    // body
                    RegularInstruction::Null,
                ),
            )],
        );
    }

    #[test]
    fn preamble_single_recursive_ref() {
        // var a = shared mut [];
        // a.0 = 'a;

        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let (owned, address) = generate_shared_owned_value(
            address_provider,
            ValueContainer::Local(List::default().into()),
            SharedContainerMutability::Mutable,
        );
        let owned_container = SharedContainer::Owned(owned);

        // self referencing list
        {
            let container_ref = SharedContainer::Referenced(
                owned_container.derive_reference_with_max_mutability(),
            );
            let mut container_mut = owned_container.value_container_mut();
            let list = container_mut.try_as_mut::<List>().unwrap();
            list.push(ValueContainer::Shared(container_ref));
        }

        assert_preamble_instructions(
            vec![(
                owned_container,
                TrackedValueMetadata::Root {
                    index: StackIndex(0),
                    is_known: false,
                },
            )],
            vec![RegularInstruction::statements_with_children(
                false,
                instructions!(
                    // preamble
                    RegularInstruction::PushListToStack,
                    RegularInstruction::statements_with_children(
                        false,
                        instructions!(
                            // shared mut a = [null]
                            RegularInstruction::PushToStack,
                            RegularInstruction::MoveWithValue(MoveWithValue {
                                mutability: SharedContainerMutability::Mutable,
                                previous_address: address.clone(),
                            })
                            .with_children(
                                instructions!(
                                    RegularInstruction::ShortList(
                                        ShortListData { element_count: 1 }
                                    )
                                    .with_children(instructions!(
                                        RegularInstruction::Null
                                    ))
                                )
                            ),
                            // ('mut a).0 = 'mut a
                            RegularInstruction::PushToStack,
                            RegularInstruction::GetStackValueSharedRefMut(
                                StackIndex(0)
                            ),
                            RegularInstruction::SetEntryIndex(UInt32Data(0))
                                .with_children(instructions!(
                                    RegularInstruction::BorrowStackValue(
                                        StackIndex(1)
                                    ),
                                    RegularInstruction::BorrowStackValue(
                                        StackIndex(1)
                                    )
                                )),
                            RegularInstruction::list_with_children(
                                instructions!(
                                    RegularInstruction::TakeStackValue(
                                        StackIndex(0)
                                    )
                                )
                            ),
                        )
                    ),
                    // body
                    RegularInstruction::Null
                ),
            )],
        );
    }

    #[test]
    fn preamble_nested_recursive_ref() {
        // var x = shared {a: {b: null}};
        // x.a.b = 'x;

        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let (owned, address) = generate_shared_owned_value(
            address_provider,
            ValueContainer::Local(Map::default().into()),
            SharedContainerMutability::Mutable,
        );
        let owned_container = SharedContainer::Owned(owned);

        // self referencing map
        {
            let container_ref = SharedContainer::Referenced(
                owned_container.derive_reference_with_max_mutability(),
            );
            let mut container_mut = owned_container.value_container_mut();
            let map = container_mut.try_as_mut::<Map>().unwrap();
            map.set_unchecked(
                "a",
                Map::from(vec![(
                    ValueContainer::from("b"),
                    ValueContainer::Shared(container_ref),
                )]),
            );
        }

        assert_preamble_instructions(
            vec![(
                owned_container,
                TrackedValueMetadata::Root {
                    index: StackIndex(0),
                    is_known: false,
                },
            )],
            vec![RegularInstruction::statements_with_children(
                false,
                instructions!(
                    // preamble
                    RegularInstruction::PushListToStack,
                    RegularInstruction::statements_with_children(
                        false,
                        instructions!(
                            RegularInstruction::PushToStack,
                            RegularInstruction::MoveWithValue(MoveWithValue {
                                mutability: SharedContainerMutability::Mutable,
                                previous_address: address.clone(),
                            })
                            .with_children(instructions!(
                                RegularInstruction::ShortMap(ShortMapData { element_count: 1 }).with_children(instructions!(
                                    RegularInstruction::KeyValueShortText(ShortTextData("a".to_string())).with_children(
                                        instructions!(
                                             RegularInstruction::ShortMap(
                                                ShortMapData { element_count: 1 }
                                            )
                                            .with_children(instructions!(
                                                RegularInstruction::KeyValueShortText(ShortTextData("b".to_string())).with_children(
                                                    instructions!(
                                                        RegularInstruction::Null
                                                    )
                                                )
                                            ))
                                        )
                                    ),

                                ))
                            )),

                            RegularInstruction::PushToStack,
                            RegularInstruction::GetStackValueSharedRefMut(StackIndex(0)),

                            RegularInstruction::SetEntryDynamic.with_children(
                                instructions!(
                                    RegularInstruction::ShortText(ShortTextData("b".to_string())),
                                    RegularInstruction::BorrowStackValue(StackIndex(1)),
                                    RegularInstruction::GetEntryDynamic.with_children(
                                        instructions!(
                                            RegularInstruction::ShortText(ShortTextData("a".to_string())),
                                            RegularInstruction::BorrowStackValue(StackIndex(1))
                                        )
                                    )
                                )
                            ),
                            RegularInstruction::list_with_children(
                                instructions!(
                                    RegularInstruction::TakeStackValue(
                                        StackIndex(0)
                                    )
                                )
                            )
                        )
                    ),
                    // body
                    RegularInstruction::Null
                ),
            )],
        );
    }

    #[test]
    fn preamble_triple_ref() {
        // var a = shared {b: null};
        // var b = shared {c: null};
        // a.b = 'b;
        // var c = shared {a: a};
        // b.c = c;

        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let (owned_container_a, address_a) = generate_shared_owned_value(
            address_provider,
            ValueContainer::Local(Map::default().into()),
            SharedContainerMutability::Mutable,
        );
        let (owned_container_b, address_b) = generate_shared_owned_value(
            address_provider,
            ValueContainer::Local(Map::default().into()),
            SharedContainerMutability::Mutable,
        );
        let (owned_container_c, address_c) = generate_shared_owned_value(
            address_provider,
            ValueContainer::Local(
                Map::from(vec![(
                    ValueContainer::from("a"),
                    ValueContainer::Shared(SharedContainer::Referenced(
                        owned_container_a.derive_with_max_mutability(),
                    )),
                )])
                .into(),
            ),
            SharedContainerMutability::Mutable,
        );
        // set a.b = b;
        {
            let mut container_mut = owned_container_a.value_container_mut();
            let map = container_mut.try_as_mut::<Map>().unwrap();
            map.set_unchecked(
                "b",
                ValueContainer::Shared(SharedContainer::Referenced(
                    owned_container_b.derive_with_max_mutability(),
                )),
            );
        }
        // set b.c = c;
        {
            let mut container_mut = owned_container_b.value_container_mut();
            let map = container_mut.try_as_mut::<Map>().unwrap();
            map.set_unchecked(
                "c",
                ValueContainer::Shared(SharedContainer::Referenced(
                    owned_container_c.derive_with_max_mutability(),
                )),
            );
        }

        assert_preamble_instructions(
            vec![
                (
                    SharedContainer::Referenced(
                        owned_container_c.derive_with_max_mutability(),
                    ),
                    TrackedValueMetadata::Child { is_known: false },
                ),
                (
                    SharedContainer::Referenced(
                        owned_container_b.derive_with_max_mutability(),
                    ),
                    TrackedValueMetadata::Child { is_known: false },
                ),
                (
                    SharedContainer::Owned(owned_container_a),
                    TrackedValueMetadata::Root {
                        index: StackIndex(0),
                        is_known: false,
                    },
                ),
            ],
            vec![RegularInstruction::statements_with_children(
                false,
                instructions!(
                    // preamble
                    RegularInstruction::PushListToStack,
                    RegularInstruction::statements_with_children(
                        false,
                        instructions!(
                            RegularInstruction::PushToStack,
                            RegularInstruction::MoveWithValue(MoveWithValue {
                                mutability: SharedContainerMutability::Mutable,
                                previous_address: address_a.clone(),
                            })
                            .with_children(
                                instructions!(
                                    RegularInstruction::ShortMap(
                                        ShortMapData { element_count: 1 }
                                    )
                                    .with_children(instructions!(
                                        RegularInstruction::KeyValueShortText(
                                            ShortTextData("b".to_string())
                                        )
                                        .with_children(instructions!(
                                            RegularInstruction::Null
                                        ))
                                    ))
                                )
                            ),
                            RegularInstruction::PushToStack,
                            RegularInstruction::GetStackValueSharedRefMut(
                                StackIndex(0)
                            ),
                            RegularInstruction::PushToStack,
                            RegularInstruction::SharedRefWithValue(
                                SharedRefWithValue {
                                    ref_mutability:
                                        ReferenceMutability::Mutable,
                                    address: address_b.clone(),
                                    container_mutability:
                                        SharedContainerMutability::Mutable,
                                }
                            )
                            .with_children(
                                instructions!(
                                    RegularInstruction::ShortMap(
                                        ShortMapData { element_count: 1 }
                                    )
                                    .with_children(instructions!(
                                        RegularInstruction::KeyValueShortText(
                                            ShortTextData("c".to_string())
                                        )
                                        .with_children(instructions!(
                                            RegularInstruction::Null
                                        ))
                                    ))
                                )
                            ),
                            RegularInstruction::SetEntryDynamic.with_children(
                                instructions!(
                                    RegularInstruction::ShortText(
                                        ShortTextData("b".to_string())
                                    ),
                                    RegularInstruction::BorrowStackValue(
                                        StackIndex(2)
                                    ),
                                    RegularInstruction::BorrowStackValue(
                                        StackIndex(1)
                                    )
                                )
                            ),
                            RegularInstruction::PushToStack,
                            RegularInstruction::SharedRefWithValue(
                                SharedRefWithValue {
                                    ref_mutability:
                                        ReferenceMutability::Mutable,
                                    address: address_c.clone(),
                                    container_mutability:
                                        SharedContainerMutability::Mutable,
                                }
                            )
                            .with_children(
                                instructions!(
                                    RegularInstruction::ShortMap(
                                        ShortMapData { element_count: 1 }
                                    )
                                    .with_children(instructions!(
                                        RegularInstruction::KeyValueShortText(
                                            ShortTextData("a".to_string())
                                        )
                                        .with_children(instructions!(
                                        RegularInstruction::BorrowStackValue(
                                            StackIndex(1)
                                        )
                                    ))
                                    ))
                                )
                            ),
                            RegularInstruction::SetEntryDynamic.with_children(
                                instructions!(
                                    RegularInstruction::ShortText(
                                        ShortTextData("c".to_string())
                                    ),
                                    RegularInstruction::BorrowStackValue(
                                        StackIndex(3)
                                    ),
                                    RegularInstruction::BorrowStackValue(
                                        StackIndex(2)
                                    )
                                )
                            ),
                            RegularInstruction::list_with_children(
                                instructions!(
                                    RegularInstruction::TakeStackValue(
                                        StackIndex(0)
                                    )
                                )
                            )
                        )
                    ),
                    // body
                    RegularInstruction::Null
                ),
            )],
        );
    }
}
