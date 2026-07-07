use crate::{
    global::protocol_structures::instruction_data::StackIndex,
    prelude::*,
    random::RandomState,
    runtime::pointer_availability_lookup::PointerAvailabilityLookup,
    shared_values::{
        SharedContainer, traits::SharedContainerCommon,
    },
    traits::child_iterator::ChildIterator,
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};
use core::ops::DerefMut;
use indexmap::IndexMap;

#[derive(Debug)]
pub enum TrackedValueMetadata {
    Root {
        index: StackIndex,
        /// if the pointer of this container is already known for to be available on all receivers
        is_known: bool,
    },
    Child {
        /// if the pointer of this container is already known for to be available on all receivers
        is_known: bool,
    },
}

impl TrackedValueMetadata {
    pub fn is_known(&self) -> bool {
        match self {
            TrackedValueMetadata::Root { is_known, .. } => *is_known,
            TrackedValueMetadata::Child { is_known, .. } => *is_known,
        }
    }
}

/// Helper struct used during compilation to keep track which shared values are moved or referenced
#[derive(Debug)]
pub struct SharedValueTracking<'a> {
    /// shared values that were injected in the compiler
    pub tracked_values:
        IndexMap<SharedContainer, TrackedValueMetadata, RandomState>,
    pub current_stack_index: StackIndex,
    pub pointer_availability_lookup: &'a PointerAvailabilityLookup,
    pub receivers: &'a [Endpoint],
}

/// # Safety: This function is unsafe because it leaks memory and returns a reference with a static lifetime. It should only be used in tests where the leaked memory is acceptable.
pub(crate) unsafe fn default_tracking<'a>() -> SharedValueTracking<'a> {
    let lookup = Box::leak(Box::new(PointerAvailabilityLookup::default()));
    let receivers: &'a [Endpoint] = Box::leak(Box::new([]));
    SharedValueTracking::new(lookup, receivers)
}

#[derive(Debug, Default)]
pub struct TrackedValueCollection {
    pub tracked_values: Vec<(SharedContainer, TrackedValueMetadata)>,
    pub root_count: usize,
}

impl TrackedValueCollection {
    pub fn has_tracked_values(&self) -> bool {
        self.root_count > 0
    }
}

impl<'a> SharedValueTracking<'a> {
    pub fn new(
        pointer_availability_lookup: &'a PointerAvailabilityLookup,
        receivers: &'a [Endpoint],
    ) -> SharedValueTracking<'a> {
        SharedValueTracking {
            tracked_values: IndexMap::default(),
            current_stack_index: StackIndex(0),
            pointer_availability_lookup,
            receivers,
        }
    }

    fn get_next_stack_index(&mut self) -> StackIndex {
        let stack_index = self.current_stack_index;
        self.current_stack_index = StackIndex(self.current_stack_index.0 + 1);
        stack_index
    }

    /// Updates the tracked value for a shared container if the new container has higher ownership than the existing one
    /// Returns the passed container if it was not yet in the tracked values
    fn update_container_ownership_if_exists(
        &mut self,
        container: SharedContainer,
    ) -> Option<SharedContainer> {
        if let Some((existing, _)) =
            self.tracked_values.get_key_value(&container)
        {
            if container.ownership() > existing.ownership() {
                let index =
                    self.tracked_values.get_index_of(&container).unwrap();
                self.tracked_values.replace_index(index, container).unwrap();
            }
            None
        } else {
            Some(container)
        }
    }

    /// Registers a new shared value. Returns a stack index that can be used to access this value
    pub fn register_shared_value(
        &mut self,
        shared_container: SharedContainer,
    ) -> StackIndex {
        let shared_container_clone = shared_container.clone();

        self.register_shared_value_with_parents(
            shared_container,
            &mut HashSet::new(),
        );

        // ensure tracked value is a top level tracked value with stack index
        match self.tracked_values.get(&shared_container_clone).unwrap() {
            TrackedValueMetadata::Child { is_known, .. } => {
                let is_known = *is_known;
                let index = self.get_next_stack_index();
                let tracked_value = self
                    .tracked_values
                    .get_mut(&shared_container_clone)
                    .unwrap();
                *tracked_value = TrackedValueMetadata::Root { index, is_known };
                index
            }
            // already a top level value, do nothing
            TrackedValueMetadata::Root { index, .. } => *index,
        }
    }

    fn register_shared_value_with_parents(
        &mut self,
        shared_container: SharedContainer,
        parents: &mut HashSet<SharedContainer>,
    ) {
        // already registered as referenced, update the container mutability if needed
        if let Some(container) =
            self.update_container_ownership_if_exists(shared_container)
        {
            let is_known = !self.receivers.is_empty()
                && self
                    .pointer_availability_lookup
                    .is_available_for_all_endpoints(
                        self.receivers,
                        &container.pointer_address(),
                    );

            let parent_moved = container.treat_as_move();
            let container_clone = container.clone();

            self.tracked_values
                .insert(container, TrackedValueMetadata::Child { is_known });

            // If the address is not already being visited, we want to register all children
            // with the whole tree of their direct and indirect parents
            if parents.insert(container_clone.clone())
                && (!is_known || parent_moved)
            {
                let mut inner_container = container_clone.value_container_mut();
                match inner_container.deref_mut() {
                    ValueContainer::Shared(inner_shared) => {
                        self.register_child(
                            parent_moved,
                            inner_shared,
                            parents,
                        );
                    }
                    _ => {
                        inner_container.with_collapsed_value_mut(|value| {
                            for child in value.iter_children_mut() {
                                if let ValueContainer::Shared(child) = child {
                                    self.register_child(
                                        parent_moved,
                                        child,
                                        parents,
                                    );
                                }
                            }
                        });
                    }
                }

                parents.remove(&container_clone);
            }
        }
    }

    fn register_child(
        &mut self,
        parent_moved: bool,
        child: &mut SharedContainer,
        parents: &mut HashSet<SharedContainer>,
    ) {
        // If the parent is moved and the child is owned, we must also move the child
        // if parent is not moved, the child is also not moved, even if it is owned by the parent
        if parent_moved {
            self.register_shared_value_with_parents(
                // Note we can convert to a ref here since the parent
                // was already a ref, so the child can never be owned
                child.downgrade_to_reference(),
                parents,
            );
        } else {
            self.register_shared_value_with_parents(
                // Note we can convert to a ref here since the parent
                // was already a ref, so the child can never be owned
                child.clone(),
                parents,
            );
        }
    }

    /// Extracts all registered owned and referenced shared values
    pub fn into_tracked_values(self) -> TrackedValueCollection {
        TrackedValueCollection {
            tracked_values: self.tracked_values.into_iter().collect(),
            root_count: self.current_stack_index.0 as usize,
        }
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
        values::{core_values::list::List, value_container::ValueContainer},
    };
    use core::assert_matches;

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
    ) -> &'a TrackedValueMetadata {
        tracking
            .tracked_values
            .get(container)
            .expect("expected shared value to be tracked")
    }

    fn assert_top_level(
        tracking: &SharedValueTracking,
        container: &SharedContainer,
        expected_index: StackIndex,
    ) {
        match tracked_value(tracking, container) {
            TrackedValueMetadata::Root { index, .. } => {
                assert_eq!(*index, expected_index);
            }
            TrackedValueMetadata::Child { .. } => {
                panic!("expected top-level tracked value")
            }
        }
    }

    fn assert_child(
        tracking: &SharedValueTracking,
        container: &SharedContainer,
    ) {
        assert_matches!(
            tracked_value(tracking, container),
            &TrackedValueMetadata::Child { .. }
        );
    }

    fn tracking() -> SharedValueTracking<'static> {
        unsafe { default_tracking() }
    }

    #[test]
    fn index_start_at_one() {
        let tracking = tracking();
        assert_eq!(tracking.tracked_values.len(), 0);
        assert_eq!(tracking.current_stack_index, StackIndex(0));
    }

    #[test]
    fn single_toplevel() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let (container, _) = owned_shared(
            address_provider,
            42,
            SharedContainerMutability::Immutable,
        );

        let lookup = &PointerAvailabilityLookup::default();
        let mut tracking = tracking();

        let index = tracking.register_shared_value(container.clone());

        assert_eq!(index, StackIndex(0));
        assert_eq!(tracking.tracked_values.len(), 1);

        assert_top_level(&tracking, &container, StackIndex(0));
    }

    #[test]
    fn top_level_reuse_index() {
        let mut tracking = tracking();
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let (container, _) = owned_shared(
            address_provider,
            42,
            SharedContainerMutability::Immutable,
        );

        let first_index = tracking.register_shared_value(container.clone());
        let second_index = tracking.register_shared_value(container.clone());
        assert_eq!(first_index, StackIndex(0));
        assert_eq!(second_index, StackIndex(0));
        assert_eq!(tracking.tracked_values.len(), 1);

        assert_top_level(&tracking, &container, StackIndex(0));
    }

    #[test]
    fn two_toplevel() {
        let mut tracking = tracking();
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

        assert_eq!(first_index, StackIndex(0));
        assert_eq!(second_index, StackIndex(1));
        assert_eq!(tracking.tracked_values.len(), 2);

        assert_top_level(&tracking, &first, StackIndex(0));
        assert_top_level(&tracking, &second, StackIndex(1));
    }

    #[test]
    fn direct_shared_child_list() {
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

        let mut tracking = tracking();
        let parent_index = tracking.register_shared_value(parent.clone());

        assert_eq!(parent_index, StackIndex(0));
        assert_eq!(tracking.tracked_values.len(), 2);

        assert_top_level(&tracking, &parent, StackIndex(0));
        assert_child(&tracking, &child);
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

        let child_clone = child.clone();

        // parent = shared child
        let (parent, _) = owned_shared(
            address_provider,
            ValueContainer::Shared(child),
            SharedContainerMutability::Immutable,
        );
        let parent_clone = parent.clone();

        let mut tracking = tracking();
        let parent_index = tracking.register_shared_value(parent);

        assert_eq!(parent_index, StackIndex(0));
        assert_eq!(tracking.tracked_values.len(), 2);

        assert_top_level(&tracking, &parent_clone, StackIndex(0));
        assert_child(&tracking, &child_clone);
    }

    #[test]
    fn child_get_top_level() {
        let mut tracking = tracking();
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

        assert_eq!(parent_index, StackIndex(0));
        assert_eq!(child_index, StackIndex(1));
        assert_eq!(tracking.tracked_values.len(), 2);

        assert_top_level(&tracking, &parent, StackIndex(0));
        assert_top_level(&tracking, &child, StackIndex(1));
    }

    #[test]
    fn tracking_same_child_tracked_once() {
        let mut tracking = tracking();

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

        assert_eq!(first_parent_index, StackIndex(0));
        assert_eq!(second_parent_index, StackIndex(1));
        assert_eq!(tracking.tracked_values.len(), 3);

        assert_top_level(&tracking, &first_parent, StackIndex(0));
        assert_top_level(&tracking, &second_parent, StackIndex(1));
        assert_child(&tracking, &child);
    }

    #[test]
    fn self_referencing() {
        let mut tracking = tracking();
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

        assert_eq!(parent_index, StackIndex(0));
        assert_eq!(tracking.tracked_values.len(), 1);

        assert_top_level(&tracking, &parent, StackIndex(0));
    }
}
