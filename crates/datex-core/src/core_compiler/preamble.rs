use crate::{
    collections::HashMap,
    core_compiler::{
        buffer_provider::BufferProvider,
        core_compilation_context::{ByteCursor, CoreCompilationContext},
        shared_value_tracking::{TrackedValue, TrackedValueCollection},
        value_compiler::{
            append_inline_shared_container, append_regular_instruction,
            append_value,
        },
        value_visitor::ValueVisitor,
    },
    datex_proxy::DatexValueContainerProxyInfallibleSerialize,
    global::protocol_structures::{
        instruction_data::{
            ListData, SharedRef, SharedRefWithValue, ShortListData, StackIndex,
        },
        regular_instructions::RegularInstruction,
    },
    shared_values::{PointerAddress, SharedContainer},
    types::r#type::Type,
    values::value_container::{ValueContainer, value_key::ValueKey},
};
use binrw::io::Write;

#[derive(Debug)]
enum VisitedValue {
    /// Indicates that the shared container was already inserted and is available at the given stack index
    Inserted { stack_index: StackIndex },
    /// Indicates that the shared container is referenced by one or multiple shared containers that were already inserted
    Required {
        partial_instantiations: Vec<DependantPartialInstantiations>,
    },
}

#[derive(Debug)]
struct DependantPartialInstantiations {
    target_index: StackIndex,
    assigned_property: ValueKey, // TODO: also support direct ref ("newtype" struct) assignments
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

impl ValueVisitor for PreambleContext<'_> {
    fn visit_value_container(&mut self, value_container: ValueContainer) {
        match value_container {
            ValueContainer::Local(value) => append_value(self, value),
            ValueContainer::Shared(shared_container) => {
                match self.visited_values.get_mut(&shared_container) {
                    // shared container was not yet inserted, keep track as required dependency
                    None => {
                        todo!();
                    }
                    Some(VisitedValue::Required {
                        partial_instantiations,
                    }) => {
                        todo!()
                    }
                    // shared container was already inserted, use stack value
                    Some(VisitedValue::Inserted { stack_index }) => {
                        append_regular_instruction(
                            self.cursor,
                            RegularInstruction::BorrowStackValue(*stack_index),
                        );
                    }
                }
            }
        }
    }

    fn visit_type(&mut self, ty: Type) {
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
    let mut byte_cursor = ByteCursor::new(vec![]);
    let tracked_values = collection.tracked_values;
    // no injected values
    if tracked_values.is_empty() {
        return (body, vec![]);
    }

    let context = &mut PreambleContext {
        cursor: &mut byte_cursor,
        visited_values: HashMap::new(),
        current_stack_index: StackIndex(0),
    };
    let mut root_containers =
        Vec::<SharedContainer>::with_capacity(collection.root_count);

    // statements (preamble + body) start
    append_regular_instruction(
        context.cursor,
        RegularInstruction::statements(2, false),
    );

    // spread preamble injected value
    append_regular_instruction(
        context.cursor,
        RegularInstruction::PushListToStack,
    );

    // statements (injected values [n] + short list [1])
    append_regular_instruction(
        context.cursor,
        RegularInstruction::statements(1 + tracked_values.len() as u32, false),
    );

    let mut root_container_stack_indices: Vec<Option<StackIndex>> =
        vec![None; collection.root_count];

    // loop over all injected values
    for tracked_value in tracked_values.into_iter().rev() {
        let index = append_injected_value(context, &tracked_value);
        if let TrackedValue::Root {
            index: root_stack_index,
            ..
        } = tracked_value
        {
            // if it is a root tracked value, register in the root container list
            root_container_stack_indices[root_stack_index.0 as usize] =
                Some(index);
            root_containers.push(tracked_value.into_container());
        }
    }
    // asserting that root stack index keys are a contiguous list of 0-n -> inner stack indices
    let root_container_stack_indices_sorted = root_container_stack_indices
        .iter()
        .map(|opt| opt.expect("Root stack index should have been registered"))
        .collect::<Vec<_>>();

    // append [#0,...#x]
    append_regular_instruction(
        context.cursor,
        RegularInstruction::list(
            root_container_stack_indices_sorted.len() as u32
        ),
    );

    for stack_index in root_container_stack_indices_sorted.iter() {
        append_regular_instruction(
            context.cursor,
            RegularInstruction::TakeStackValue(*stack_index),
        );
    }

    // append body
    context.cursor.write_all(&body).unwrap();

    (byte_cursor.into_inner(), root_containers)

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
    tracked_value: &TrackedValue,
) -> StackIndex {
    match context.visited_values.get(tracked_value.container()) {
        // value was not yet required or inserted, just add compilation to push new to stack
        None => push_injected_value(context, tracked_value.container()),
        // value is required by other shared container that was already inserted,
        // first push value, then init missing partial instantiations
        Some(VisitedValue::Required {
            partial_instantiations,
        }) => {
            // first push the value
            let container = tracked_value.container();
            push_injected_value(context, container);
            // once instantiated, also init missing partial instantiations
            todo!()
        }
        // this should never happen since all shared values are registered in tracked_values
        // exactly once
        Some(VisitedValue::Inserted { .. }) => {
            unreachable!("Shared container already inserted");
        }
    }
}

fn push_injected_value(
    context: &mut PreambleContext,
    container: &SharedContainer,
) -> StackIndex {
    let index = context.get_next_stack_index();
    let container_clone = container.clone();

    // append push to stack
    append_regular_instruction(context.cursor, RegularInstruction::PushToStack);

    match container {
        SharedContainer::Referenced(referenced_container) => {
            match referenced_container.pointer_address() {
                // insert with value for self owned references
                PointerAddress::SelfOwned(pointer_address) => {
                    append_regular_instruction(
                        context.cursor,
                        RegularInstruction::SharedRefWithValue(
                            SharedRefWithValue {
                                address: pointer_address.into(),
                                ref_mutability: referenced_container
                                    .reference_mutability(),
                                container_mutability: referenced_container
                                    .container_mutability(),
                            },
                        ),
                    );
                    // TODO: no clone?
                    context.visit_value_container(
                        referenced_container.value_container().clone(),
                    );
                }
                // insert without value for non self owned references
                PointerAddress::Remote(pointer_address) => {
                    append_regular_instruction(
                        context.cursor,
                        RegularInstruction::SharedRef(SharedRef {
                            address: PointerAddress::Remote(pointer_address)
                                .into(),
                            ref_mutability: referenced_container
                                .reference_mutability(),
                            container_mutability: referenced_container
                                .container_mutability(),
                        }),
                    );
                }
            }
        }

        SharedContainer::Owned(owned_container) => {
            todo!("move")
        }
    }
    // register as inserted value
    context.visited_values.insert(
        container_clone,
        VisitedValue::Inserted { stack_index: index },
    );

    index
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_regular_instructions_equal,
        core_compiler::{
            core_compilation_context::ByteCursor,
            preamble::append_injected_values_preamble,
            shared_value_tracking::{TrackedValue, TrackedValueCollection},
            value_compiler::append_regular_instruction,
        },
        disassembler::{
            disassemble_body_to_string, options::DisassemblerOptions,
        },
        global::protocol_structures::{
            instruction_data::{
                Int32Data, ListData, SharedRefWithValue, ShortListData,
                StackIndex,
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
        // mock body
        let mut cursor = ByteCursor::new(vec![]);
        append_regular_instruction(&mut cursor, RegularInstruction::Null);
        let collection = TrackedValueCollection {
            root_count: shared_containers
                .iter()
                .filter(|v| matches!(v, TrackedValue::Root { .. }))
                .count(),
            tracked_values: shared_containers,
        };
        let (bytes, _) =
            append_injected_values_preamble(collection, cursor.into_inner());

        println!(
            "{}",
            disassemble_body_to_string(
                &bytes,
                DisassemblerOptions {
                    tree: true,
                    colorized: true,
                    recursive: false,
                }
            )
        );

        assert_regular_instructions_equal!(&bytes, instructions);
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
                index: StackIndex(0),
            }],
            vec![
                RegularInstruction::statements(2, false),
                // preamble
                RegularInstruction::PushListToStack,
                RegularInstruction::statements(2, false),
                // ref
                RegularInstruction::PushToStack,
                RegularInstruction::SharedRefWithValue(SharedRefWithValue {
                    address: address.into(),
                    ref_mutability: ReferenceMutability::Immutable,
                    container_mutability: SharedContainerMutability::Immutable,
                }),
                RegularInstruction::Int32(Int32Data(42)),
                RegularInstruction::ShortList(ShortListData {
                    element_count: 1,
                }),
                RegularInstruction::TakeStackValue(StackIndex(0)),
                // body
                RegularInstruction::Null,
            ],
        );
    }
}
