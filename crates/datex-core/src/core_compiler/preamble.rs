use crate::{
    collections::HashMap,
    core_compiler::{
        buffer_provider::BufferProvider,
        core_compilation_context::ByteCursor,
        shared_value_tracking::{TrackedValueCollection, TrackedValueMetadata},
        value_compiler::{append_regular_instruction, append_value},
        value_visitor::{ParentAccessor, ParentContext, ValueVisitor},
    },
    global::protocol_structures::{
        instruction_data::{
            MoveWithValue, SharedRef, SharedRefWithValue, StackIndex,
            UInt32Data,
        },
        regular_instructions::RegularInstruction,
    },
    prelude::*,
    shared_values::{
        OwnedSharedContainer, PointerAddress, ReferencedSharedContainer,
        SharedContainer,
    },
    types::r#type::Type,
    values::{
        value::Value,
        value_container::{ValueContainer, value_key::ValueKey},
    },
};
use binrw::io::Write;
use crate::decompiler::{decompile_value, DecompileOptions};
use crate::shared_values::shared_container_common::SharedContainerCommon;

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
    current_stack_index: StackIndex,
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
                println!("visit with context {:?}: {}", parent_context.accessors, decompile_value(
                    &ValueContainer::Shared(parent_context.parent.clone()),
                    DecompileOptions::default(),
                ));
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
                        append_regular_instruction(
                            self.cursor,
                            RegularInstruction::Null,
                        );
                    }
                    Some(VisitedValue::Required {
                        parent_contexts: partial_instantiations,
                    }) => {
                        partial_instantiations.push(parent_context);
                        // placeholder, will be later replaced via setter
                        append_regular_instruction(
                            self.cursor,
                            RegularInstruction::Null,
                        );
                    }
                    // shared container has already been inserted, use stack value
                    // depending on the move flag, we either take the value or borrow it
                    Some(VisitedValue::Inserted { stack_index }) => {
                        // append_regular_instruction(
                        //     self.cursor,
                        //     RegularInstruction::PushToStack,
                        // );

                        if shared_container.treat_as_move() {
                            append_regular_instruction(
                                self.cursor,
                                RegularInstruction::TakeStackValue(
                                    *stack_index,
                                ),
                            );
                        } else {
                            append_regular_instruction(
                                self.cursor,
                                RegularInstruction::BorrowStackValue(
                                    *stack_index,
                                ),
                            );
                        }
                    }
                }
            }
        }
    }

    fn visit_type(&mut self, _ty: Type) {
        todo!()
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
    let mut inner_statement_count = 0u32;

    {
        let context = &mut PreambleContext {
            cursor: &mut inner_cursor,
            visited_values: HashMap::new(),
            current_stack_index: StackIndex(0),
        };

        // loop through the tracked values in reverse order (as optimization)
        for (tracked_value, metadata) in tracked_values.into_iter().rev() {
            let (index, patch_count) =
                append_injected_value(context, &tracked_value, &metadata);

            // push + set propetyr instruction
            inner_statement_count += 1 + patch_count;

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
        append_regular_instruction(
            context.cursor,
            RegularInstruction::list(
                root_container_stack_indices_sorted.len() as u32
            ),
        );

        // insert all root stack values into the list, which is the final value of the preamble
        // append [#0,...#x]
        for stack_index in root_container_stack_indices_sorted {
            append_regular_instruction(
                context.cursor,
                RegularInstruction::TakeStackValue(stack_index),
            );
        }

        // The final root list is also one inner statement.
        inner_statement_count += 1;
    }
    let mut byte_cursor = ByteCursor::new(vec![]);

    // statements (preamble + body) start
    append_regular_instruction(
        &mut byte_cursor,
        RegularInstruction::statements(2, false),
    );

    // the preamble pushes a list of the root shared containers to the stack
    append_regular_instruction(
        &mut byte_cursor,
        RegularInstruction::PushListToStack,
    );

    // the inner preamble statements, which push the shared containers to the stack
    // and apply the patches
    append_regular_instruction(
        &mut byte_cursor,
        RegularInstruction::statements(inner_statement_count, false),
    );
    // the inner preamble content, which pushes the shared containers to the stack and applies the patches
    byte_cursor.write_all(&inner_cursor.into_inner()).unwrap();

    // append body
    byte_cursor.write_all(&body).unwrap();
    (byte_cursor.into_inner(), root_containers)
}

fn tracking_key(container: &SharedContainer) -> SharedContainer {
    SharedContainer::Referenced(
        container.derive_reference_with_max_mutability(),
    )
}

fn append_injected_value(
    context: &mut PreambleContext,
    container: &SharedContainer,
    metadata: &TrackedValueMetadata,
) -> (StackIndex, u32) /* the patches count to calc statement count */ {
    let key = tracking_key(container);

    let existing_parent_contexts = match context.visited_values.remove(&key) {
        None => Vec::new(),
        Some(VisitedValue::Required { parent_contexts }) => parent_contexts,
        Some(VisitedValue::Inserted { .. }) => {
            unreachable!("Shared container already inserted")
        }
    };

    context.visited_values.insert(
        key.clone(),
        VisitedValue::Required {
            parent_contexts: existing_parent_contexts,
        },
    );

    let index =
        push_injected_container(context, container, metadata.is_known());

    let pending_contexts = match context.visited_values.remove(&key) {
        Some(VisitedValue::Required { parent_contexts }) => parent_contexts,
        Some(VisitedValue::Inserted { .. }) => {
            unreachable!("Container was marked inserted while still compiling")
        }
        None => {
            unreachable!(
                "Required tracking entry disappeared during compilation"
            )
        }
    };

    context
        .visited_values
        .insert(key, VisitedValue::Inserted { stack_index: index });
    let patch_count = pending_contexts.len() as u32;

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
        println!(
            "deferred assignment: parent {} at stack index {:?}, assigned property {}",
            parent, parent_stack_index, assigned_property
        );
        match assigned_property {
            ParentAccessor::ValueKey(ValueKey::Index(index)) => {
                append_regular_instruction(
                    context.cursor_mut(),
                    RegularInstruction::SetPropertyIndex(UInt32Data(
                        index as u32,
                    )),
                );
                // first target, or first instru?
                append_regular_instruction(
                    context.cursor,
                    RegularInstruction::BorrowStackValue(
                        parent_stack_index,
                    ),
                );
                append_property_target(context, parent_stack_index, accessors);
            }
            _ => todo!(),
        }
    }

    (index, patch_count)
}

fn append_property_target(
    context: &mut PreambleContext,
    parent_stack_index: StackIndex,
    accessors: Vec<ParentAccessor>,
) {
    // this is the last accessor, we can now set the property on the parent stack value
    // if there are more accessors, we will recurse below
    let Some((accessor, remaining)) = accessors.split_last() else {
        append_regular_instruction(
            context.cursor,
            RegularInstruction::BorrowStackValue(parent_stack_index),
        );
        return;
    };

    match accessor {
        ParentAccessor::ValueKey(ValueKey::Index(property)) => {
            append_regular_instruction(
                context.cursor,
                RegularInstruction::GetPropertyIndex(UInt32Data(
                    *property as u32,
                )),
            );
            append_property_target(
                context,
                parent_stack_index,
                remaining.to_vec(),
            );
        }
        _ => {
            todo!("Dynamic deferred property access");
        }
    }
}

fn push_injected_container(
    context: &mut PreambleContext,
    container: &SharedContainer,
    is_known: bool,
) -> StackIndex {
    let index = context.get_next_stack_index();
    let container_clone = container.clone();

    // append push to stack
    append_regular_instruction(context.cursor, RegularInstruction::PushToStack);

    // register as inserted value
    // context.visited_values.insert(
    //     container_clone,
    //     VisitedValue::Inserted { stack_index: index },
    // );

    match container {
        SharedContainer::Owned(owned_container) => {
            append_move_with_value(context, owned_container);
        }
        SharedContainer::Referenced(referenced_container) => {
            if !is_known {
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

/// Appends a move instruction to the preamble to move an owned shared containers
fn append_move_with_value(
    context: &mut PreambleContext,
    owned_container: &OwnedSharedContainer,
) {
    append_regular_instruction(
        context.cursor,
        RegularInstruction::MoveWithValue(MoveWithValue {
            mutability: owned_container.container_mutability(),
            previous_address: owned_container.pointer_address().clone(),
        }),
    );

    let inner = owned_container.value_container().clone();
    match inner {
        ValueContainer::Local(value) => append_value(
            context,
            value,
            Some(ParentContext::new(SharedContainer::Referenced(
                owned_container.derive_with_max_mutability(),
            ))),
        ),
        container @ ValueContainer::Shared(_) => {
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
            append_regular_instruction(
                context.cursor,
                RegularInstruction::SharedRef(SharedRef {
                    address: PointerAddress::SelfOwned(pointer_address),
                    ref_mutability: referenced_container.reference_mutability(),
                    container_mutability: referenced_container
                        .container_mutability(),
                }),
            );
        }
        PointerAddress::Remote(pointer_address) => {
            append_regular_instruction(
                context.cursor,
                RegularInstruction::SharedRef(SharedRef {
                    address: PointerAddress::Remote(pointer_address),
                    ref_mutability: referenced_container.reference_mutability(),
                    container_mutability: referenced_container
                        .container_mutability(),
                }),
            );
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
            append_regular_instruction(
                context.cursor,
                RegularInstruction::SharedRefWithValue(SharedRefWithValue {
                    address: pointer_address,
                    ref_mutability: referenced_container.reference_mutability(),
                    container_mutability: referenced_container
                        .container_mutability(),
                }),
            );
            // TODO: no clone?
            context.visit_value_container(
                referenced_container.value_container().clone(),
                None,
            );
        }
        // insert without value for non self owned references
        PointerAddress::Remote(pointer_address) => {
            append_regular_instruction(
                context.cursor,
                RegularInstruction::SharedRef(SharedRef {
                    address: PointerAddress::Remote(pointer_address),
                    ref_mutability: referenced_container.reference_mutability(),
                    container_mutability: referenced_container
                        .container_mutability(),
                }),
            );
        }
    }
}

#[cfg(test)]
#[cfg(feature = "disassembler")]
mod tests {
    use crate::{
        assert_regular_instructions_equal,
        core_compiler::{
            core_compilation_context::ByteCursor,
            preamble::append_injected_values_preamble,
            shared_value_tracking::{
                TrackedValueCollection, TrackedValueMetadata,
            },
            value_compiler::append_regular_instruction,
        },
        disassembler::{InstructionTree, print_disassembled},
        global::protocol_structures::{
            instruction_data::{
                Int32Data, ListData, MoveWithValue, SharedRefWithValue,
                ShortListData, StackIndex, UInt32Data,
            },
            instructions::Instruction,
            regular_instructions::RegularInstruction,
        },
        instructions,
        prelude::*,
        runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
        shared_values::{
            OwnedSharedContainer, PointerAddress, ReferenceMutability,
            ReferencedSharedContainer, SelfOwnedPointerAddress,
            SelfOwnedSharedContainer, SharedContainer,
            SharedContainerMutability, SharedContainerOwnership,
        },
        values::{core_values::list::List, value_container::ValueContainer},
    };
    use crate::shared_values::shared_container_common::SharedContainerCommon;

    fn assert_preamble_instructions(
        tracked_values: Vec<(SharedContainer, TrackedValueMetadata)>,
        instructions: Vec<impl Into<InstructionTree<Instruction>>>,
    ) {
        // mock body
        let mut cursor = ByteCursor::new(vec![]);
        append_regular_instruction(&mut cursor, RegularInstruction::Null);

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

        assert_regular_instructions_equal!(&bytes, instructions);
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
                            RegularInstruction::ShortList(ShortListData {
                                element_count: 1,
                            }),
                            RegularInstruction::TakeStackValue(StackIndex(0)),
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
        // shared mut a = [];
        // a.push('a);

        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let (owned, address) = generate_shared_owned_value(
            address_provider,
            ValueContainer::Local(List::default().into()),
            SharedContainerMutability::Mutable,
        );
        let owned_container = SharedContainer::Owned(owned);

        // self referencing list
        {
            let cloned_owned = owned_container.clone();
            let mut container_mut = owned_container.value_container_mut();
            let list = container_mut.try_as_mut::<List>().unwrap();
            list.push(ValueContainer::Shared(cloned_owned));
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
                            RegularInstruction::SetPropertyIndex(UInt32Data(0)).with_children(
                                instructions!(
                                    RegularInstruction::BorrowStackValue(
                                        StackIndex(0)
                                    ),
                                    RegularInstruction::BorrowStackValue(StackIndex(0))
                                )
                            ),
                            RegularInstruction::ShortList(ShortListData { element_count: 1 }).with_children(
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
}
