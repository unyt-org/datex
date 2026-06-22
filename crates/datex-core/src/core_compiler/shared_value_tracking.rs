use crate::{
    collections::HashMap,
    global::protocol_structures::instruction_data::StackIndex,
    prelude::*,
    shared_values::{PointerAddress, SharedContainer},
    traits::child_iterator::ChildIterator,
    values::value_container::ValueContainer,
};
#[derive(Debug)]
pub enum TrackedValue {
    Root {
        container: SharedContainer,
        index: StackIndex,
    },
    Child {
        container: SharedContainer,
    },
}

impl TrackedValue {
    fn update_container(&mut self, container: SharedContainer) {
        match self {
            TrackedValue::Root {
                container: existing,
                ..
            } => {
                if container.ownership() > existing.ownership() {
                    *existing = container;
                }
            }
            TrackedValue::Child {
                container: existing,
            } => {
                if container.ownership() > existing.ownership() {
                    *existing = container;
                }
            }
        }
    }

    pub fn container(&self) -> &SharedContainer {
        match self {
            TrackedValue::Root { container, .. } => container,
            TrackedValue::Child { container } => container,
        }
    }
    
    pub fn into_container(self) -> SharedContainer {
        match self {
            TrackedValue::Root { container, .. } => container,
            TrackedValue::Child { container } => container,
        }
    }
}

/// Helper struct used during compilation to keep track which shared values are moved or referenced
#[derive(Debug)]
pub struct SharedValueTracking {
    /// shared values that were injected in the compiler
    pub shared_values: HashMap<PointerAddress, TrackedValue>,
    pub current_stack_index: StackIndex,
}

impl Default for SharedValueTracking {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedValueTracking {
    pub fn new() -> SharedValueTracking {
        SharedValueTracking {
            shared_values: HashMap::new(),
            // start at stack index 1 - index 0 is reserved for shared value collection
            current_stack_index: StackIndex(1),
        }
    }

    fn get_next_stack_index(&mut self) -> StackIndex {
        let stack_index = self.current_stack_index;
        self.current_stack_index = StackIndex(self.current_stack_index.0 + 1);
        stack_index
    }

    /// Registers a new shared value. Returns a stack index that can be used to access this value
    pub fn register_shared_value(
        &mut self,
        shared_container: SharedContainer,
    ) -> StackIndex {
        let address = shared_container.pointer_address();
        self.register_shared_value_with_parents(
            shared_container,
            &mut HashSet::new(),
        );
        let tracked_value = self.shared_values.get(&address).unwrap();

        // ensure tracked value is a top level tracked value with stack index
        match tracked_value {
            TrackedValue::Child { .. } => {
                let index = self.get_next_stack_index();
                match self.shared_values.remove(&address) {
                    Some(TrackedValue::Child { container }) => {
                        self.shared_values.insert(
                            address,
                            TrackedValue::Root { container, index },
                        );
                    }
                    _ => unreachable!(),
                }
                index
            }
            // already a top level value, do nothing
            TrackedValue::Root { index, .. } => *index,
        }
    }

    fn register_shared_value_with_parents(
        &mut self,
        shared_container: SharedContainer,
        parents: &mut HashSet<PointerAddress>,
    ) {
        let address = shared_container.pointer_address();
        if let Some(existing) = self.shared_values.get_mut(&address) {
            existing.update_container(shared_container);
            return;
        }
        let shared_ref = shared_container.clone();
        self.shared_values.insert(
            address.clone(),
            TrackedValue::Child {
                container: shared_container,
            },
        );
        // Only for references, and if the address is not already being visited, we want to register all childrens
        // with the whole tree of their direct and indirect parents
        if matches!(shared_ref, SharedContainer::Referenced(_))
            && parents.insert(address.clone())
        {
            shared_ref.value_container().with_collapsed_value(|value| {
                for child in value.iter_children() {
                    if let ValueContainer::Shared(child) = child {
                        self.register_shared_value_with_parents(
                            child.clone(),
                            parents,
                        );
                    }
                }
            });
            parents.remove(&address);
        }
    }

    /// Extracts all registered owned and referenced shared values
    pub fn into_tracked_values(self) -> Vec<TrackedValue> {
        self.shared_values.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
        shared_values::{
            PointerAddress, ReferenceMutability, SharedContainer,
            SharedContainerMutability,
        },
        value_updates::{
            update_data::AppendEntryUpdateData, update_handler::UpdateHandler,
        },
        values::{core_values::list::List, value_container::ValueContainer},
    };

    fn owned_shared(
        address_provider: &mut SelfOwnedPointerAddressProvider,
        value: impl Into<ValueContainer>,
        mutability: SharedContainerMutability,
    ) -> (SharedContainer, PointerAddress) {
        let container = SharedContainer::new_owned_with_inferred_allowed_type(
            value,
            mutability,
            address_provider,
        );
        let address = container.pointer_address();
        (container, address)
    }

    fn referenced_shared(
        container: &SharedContainer,
        mutability: ReferenceMutability,
    ) -> SharedContainer {
        SharedContainer::Referenced(
            container
                .try_derive_reference_with_mutability(mutability)
                .expect("Can not derive reference"),
        )
    }

    fn tracked_value<'a>(
        tracking: &'a SharedValueTracking,
        container: &SharedContainer,
    ) -> &'a TrackedValue {
        let address = container.pointer_address();

        tracking
            .shared_values
            .get(&address)
            .expect("expected shared value to be tracked")
    }

    fn assert_top_level(
        tracking: &SharedValueTracking,
        container: &SharedContainer,
        expected_index: StackIndex,
    ) {
        match tracked_value(tracking, container) {
            TrackedValue::Root {
                container: tracked_container,
                index,
            } => {
                assert_eq!(*index, expected_index);
                assert_eq!(
                    tracked_container.pointer_address(),
                    container.pointer_address(),
                );
            }
            TrackedValue::Child { .. } => {
                panic!("expected top-level tracked value")
            }
        }
    }

    fn assert_child(
        tracking: &SharedValueTracking,
        container: &SharedContainer,
    ) {
        match tracked_value(tracking, container) {
            TrackedValue::Child {
                container: tracked_container,
            } => {
                assert_eq!(
                    tracked_container.pointer_address(),
                    container.pointer_address(),
                );
            }
            _ => {
                panic!("expected child tracked value")
            }
        }
    }

    #[test]
    fn index_start_at_one() {
        let tracking = SharedValueTracking::new();
        assert_eq!(tracking.shared_values.len(), 0);
        assert_eq!(tracking.current_stack_index, StackIndex(1));
    }

    #[test]
    fn single_toplevel() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let (container, _) = owned_shared(
            address_provider,
            42,
            SharedContainerMutability::Immutable,
        );

        let mut tracking = SharedValueTracking::new();

        let index = tracking.register_shared_value(container.clone());

        assert_eq!(index, StackIndex(1));
        assert_eq!(tracking.shared_values.len(), 1);

        assert_top_level(&tracking, &container, StackIndex(1));
    }

    #[test]
    fn top_level_reuse_index() {
        let mut tracking = SharedValueTracking::new();
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let (container, _) = owned_shared(
            address_provider,
            42,
            SharedContainerMutability::Immutable,
        );

        let first_index = tracking.register_shared_value(container.clone());
        let second_index = tracking.register_shared_value(container.clone());
        assert_eq!(first_index, StackIndex(1));
        assert_eq!(second_index, StackIndex(1));
        assert_eq!(tracking.shared_values.len(), 1);

        assert_top_level(&tracking, &container, StackIndex(1));
    }

    #[test]
    fn two_toplevel() {
        let mut tracking = SharedValueTracking::new();
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        // #0 = first
        let (first, _) = owned_shared(
            address_provider,
            1,
            SharedContainerMutability::Immutable,
        );

        // #1 = second
        let (second, _) = owned_shared(
            address_provider,
            2,
            SharedContainerMutability::Immutable,
        );

        let first_index = tracking.register_shared_value(first.clone());
        let second_index = tracking.register_shared_value(second.clone());

        assert_eq!(first_index, StackIndex(1));
        assert_eq!(second_index, StackIndex(2));
        assert_eq!(tracking.shared_values.len(), 2);

        assert_top_level(&tracking, &first, StackIndex(1));
        assert_top_level(&tracking, &second, StackIndex(2));
    }

    #[test]
    fn direct_shared_child() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        // child
        let (child, _) = owned_shared(
            address_provider,
            42,
            SharedContainerMutability::Immutable,
        );
        let child = referenced_shared(&child, ReferenceMutability::Immutable);

        // parent = [child]
        let (parent, _) = owned_shared(
            address_provider,
            List::from(vec![ValueContainer::Shared(child.clone())]),
            SharedContainerMutability::Immutable,
        );

        let mut tracking = SharedValueTracking::new();
        let parent_index = tracking.register_shared_value(parent.clone());

        assert_eq!(parent_index, StackIndex(1));
        assert_eq!(tracking.shared_values.len(), 2);

        assert_top_level(&tracking, &parent, StackIndex(1));
        assert_child(&tracking, &child);
    }

    #[test]
    fn child_get_top_level() {
        let mut tracking = SharedValueTracking::new();
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        // child
        let (child, _) = owned_shared(
            address_provider,
            42,
            SharedContainerMutability::Immutable,
        );
        let child = referenced_shared(&child, ReferenceMutability::Immutable);

        // parent = [child]
        let (parent, _) = owned_shared(
            address_provider,
            List::from(vec![ValueContainer::Shared(child.clone())]),
            SharedContainerMutability::Immutable,
        );

        let parent_index = tracking.register_shared_value(parent.clone());
        let child_index = tracking.register_shared_value(child.clone());

        assert_eq!(parent_index, StackIndex(1));
        assert_eq!(child_index, StackIndex(2));
        assert_eq!(tracking.shared_values.len(), 2);

        assert_top_level(&tracking, &parent, StackIndex(1));
        assert_top_level(&tracking, &child, StackIndex(2));
    }

    #[test]
    fn tracking_same_child_tracked_once() {
        let mut tracking = SharedValueTracking::new();

        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        // child
        let (child, _) = owned_shared(
            address_provider,
            42,
            SharedContainerMutability::Immutable,
        );
        let child = referenced_shared(&child, ReferenceMutability::Immutable);

        // first parent = [child]
        let (first_parent, _) = owned_shared(
            address_provider,
            List::from(vec![ValueContainer::Shared(child.clone())]),
            SharedContainerMutability::Immutable,
        );

        // second parent = [child]
        let (second_parent, _) = owned_shared(
            address_provider,
            List::from(vec![ValueContainer::Shared(child.clone())]),
            SharedContainerMutability::Immutable,
        );

        let first_parent_index =
            tracking.register_shared_value(first_parent.clone());
        let second_parent_index =
            tracking.register_shared_value(second_parent.clone());

        assert_eq!(first_parent_index, StackIndex(1));
        assert_eq!(second_parent_index, StackIndex(2));
        assert_eq!(tracking.shared_values.len(), 3);

        assert_top_level(&tracking, &first_parent, StackIndex(1));
        assert_top_level(&tracking, &second_parent, StackIndex(2));
        assert_child(&tracking, &child);
    }

    #[test]
    fn self_referencing() {
        let mut tracking = SharedValueTracking::new();

        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        // parent = [parent]
        let (parent, _) = owned_shared(
            address_provider,
            42,
            SharedContainerMutability::Immutable,
        );
        let parent = referenced_shared(&parent, ReferenceMutability::Immutable);

        parent
            .value_container_mut()
            .with_collapsed_value_mut(|value| {
                value.inner =
                    List::from(vec![ValueContainer::Shared(parent.clone())])
                        .into();
            });

        let parent_index = tracking.register_shared_value(parent.clone());

        assert_eq!(parent_index, StackIndex(1));
        assert_eq!(tracking.shared_values.len(), 1);

        assert_top_level(&tracking, &parent, StackIndex(1));
    }
}
