use std::cell::RefCell;
use std::ops::Deref;
use indexmap::IndexMap;
use crate::{
    collections::HashMap,
    global::protocol_structures::instruction_data::StackIndex,
    prelude::*,
    runtime::pointer_availability_lookup::PointerAvailabilityLookup,
    shared_values::{PointerAddress, SharedContainer},
    traits::child_iterator::ChildIterator,
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};
use crate::shared_values::{OwnedSharedContainer, ReferencedSharedContainer, SelfOwnedPointerAddress, SharedContainerInner};

#[derive(Debug)]
pub enum TrackedReference {
    Root {
        container: ReferencedSharedContainer,
        index: StackIndex,
        /// if the pointer of this container is already known for to be available on all receivers
        is_known: bool,
    },
    Child {
        container: ReferencedSharedContainer,
        /// if the pointer of this container is already known for to be available on all receivers
        is_known: bool,
    },
    /// needed for swap, do not use
    _Uninitialized
}

impl TrackedReference {

    pub fn change_to_root(&mut self, index: StackIndex) {

        let old = std::mem::replace(
            self,
            TrackedReference::_Uninitialized,
        );

        *self = match old {
            TrackedReference::Root { .. } => {
                unreachable!("Root can not be changed to root reference")
            }
            TrackedReference::Child { container, is_known } => {
                TrackedReference::Root {
                    container,
                    index,
                    is_known,
                }
            }
            TrackedReference::_Uninitialized => unreachable!()
        }
    }

    pub fn is_known(&self) -> bool {
        match self {
            TrackedReference::Root { is_known, .. } => *is_known,
            TrackedReference::Child { is_known, .. } => *is_known,
            TrackedReference::_Uninitialized => unreachable!()
        }
    }
    fn update_container(&mut self, container: ReferencedSharedContainer) {
        match self {
            TrackedReference::Root {
                container: existing,
                ..
            } => {
                if container.reference_mutability() > existing.reference_mutability() {
                    *existing = container;
                }
            }
            TrackedReference::Child {
                container: existing,
                ..
            } => {
                if container.reference_mutability() > existing.reference_mutability() {
                    *existing = container;
                }
            }
            TrackedReference::_Uninitialized => unreachable!()
        }
    }

    pub fn container(&self) -> &ReferencedSharedContainer {
        match self {
            TrackedReference::Root { container, .. } => container,
            TrackedReference::Child { container, .. } => container,
            TrackedReference::_Uninitialized => unreachable!()
        }
    }

    pub fn into_container(self) -> ReferencedSharedContainer {
        match self {
            TrackedReference::Root { container, .. } => container,
            TrackedReference::Child { container, .. } => container,
            TrackedReference::_Uninitialized => unreachable!()
        }
    }
}


#[derive(Debug)]
pub enum TrackedOwned {
    Root {
        container: OwnedSharedContainer,
        index: StackIndex,
    },
    Child {
        container: OwnedSharedContainer,
    },
    /// needed for swap, do not use
    _Uninitialized,
}

impl TrackedOwned {
    pub fn change_to_root(&mut self, index: StackIndex) {
        let old = std::mem::replace(
            self,
            TrackedOwned::_Uninitialized,
        );

        *self = match old {
            TrackedOwned::Root { .. } => {
                unreachable!("Root can not be changed to root reference")
            }
            TrackedOwned::Child { container } => {
                TrackedOwned::Root { container, index }
            }
            TrackedOwned::_Uninitialized => unreachable!()
        }
    }

    pub fn container(&self) -> &OwnedSharedContainer {
        match self {
            TrackedOwned::Root { container, .. } => container,
            TrackedOwned::Child { container } => container,
            TrackedOwned::_Uninitialized => unreachable!()
        }
    }

    pub fn into_container(self) -> OwnedSharedContainer {
        match self {
            TrackedOwned::Root { container, .. } => container,
            TrackedOwned::Child { container } => container,
            TrackedOwned::_Uninitialized => unreachable!()
        }
    }
}

/// Helper struct used during compilation to keep track which shared values are moved or referenced
#[derive(Debug)]
pub struct SharedValueTracking<'a> {
    /// shared values that were injected in the compiler
    pub referenced_values: IndexMap<PointerAddress, TrackedReference>,
    pub owned_values: IndexMap<SelfOwnedPointerAddress, TrackedOwned>,
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
    pub tracked_owned_values: Vec<TrackedOwned>,
    pub tracked_referenced_values: Vec<TrackedReference>,
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
            referenced_values: IndexMap::new(),
            owned_values: IndexMap::new(),
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

    /// Registers a new shared value. Returns a stack index that can be used to access this value
    pub fn register_shared_value(
        &mut self,
        shared_container: SharedContainer,
    ) -> StackIndex {
        let address = shared_container.pointer_address();

        match shared_container {
            SharedContainer::Owned(owned) => {
                let address = owned.pointer_address().clone();
                let tracked_value = self.register_owned_shared_value_with_parents(
                    owned,
                );

                // ensure tracked value is a top level tracked value with stack index
                match tracked_value {
                    TrackedOwned::Child { .. } => {
                        let index = self.get_next_stack_index();
                        self.owned_values.get_mut(&address).unwrap().change_to_root(index);
                        index
                    }
                    // already a top level value, do nothing
                    TrackedOwned::Root { index, .. } => *index,
                    TrackedOwned::_Uninitialized => unreachable!()
                }
            }
            SharedContainer::Referenced(referenced) => {
                self.register_referenced_shared_value_with_parents(
                    referenced,
                    &mut HashSet::new(),
                );
                let tracked_value = self.referenced_values.get_mut(&address).unwrap();

                // ensure tracked value is a top level tracked value with stack index
                match tracked_value {
                    TrackedReference::Child { .. } => {
                        let index = self.get_next_stack_index();
                        self.referenced_values.get_mut(&address).unwrap().change_to_root(index);
                        index
                    }
                    // already a top level value, do nothing
                    TrackedReference::Root { index, .. } => *index,
                    TrackedReference::_Uninitialized => unreachable!()
                }
            }
        }
    }
    fn register_owned_shared_value_with_parents(
        &mut self,
        owned_container: OwnedSharedContainer,
    ) -> &mut TrackedOwned {
        // if not already registered as owned, add
        let address = owned_container.pointer_address().clone();
        self.owned_values.entry(address).or_insert(
            TrackedOwned::Child {
                container: owned_container,
            },
        )
    }

    fn register_referenced_shared_value_with_parents(
        &mut self,
        referenced_container: ReferencedSharedContainer,
        parents: &mut HashSet<PointerAddress>,
    ) {
        let address = referenced_container.pointer_address();

        // already registered as referenced, update the container mutability if needed
        if let Some(existing) = self.referenced_values.get_mut(&address) {
            existing.update_container(referenced_container);
        }
        else {
            let is_known = !self.receivers.is_empty()
                && self
                .pointer_availability_lookup
                .is_available_for_all_endpoints(self.receivers, &address);

            // store container in referenced_values
            self.referenced_values.insert(
                address.clone(),
                TrackedReference::Child {
                    container: referenced_container.clone(),
                    is_known,
                },
            );

            // If the address is not already being visited, we want to register all children
            // with the whole tree of their direct and indirect parents
            if parents.insert(address.clone()) && !is_known
            {
                referenced_container.value_container().with_collapsed_value(|value| {
                    for child in value.iter_children() {
                        if let ValueContainer::Shared(child) = child {
                            self.register_referenced_shared_value_with_parents(
                                // Note we can convert to a ref here since the parent
                                // was already a ref, so the child can never be owned
                                child.derive_with_max_mutability(),
                                parents,
                            );
                        }
                    }
                });
                parents.remove(&address);
            }
        }
    }

    /// Extracts all registered owned and referenced shared values
    pub fn into_tracked_values(self) -> TrackedValueCollection {
        TrackedValueCollection {
            tracked_owned_values: self.owned_values.into_values().collect(),
            tracked_referenced_values: self.referenced_values.into_values().collect(),
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
    ) -> &'a TrackedReference {
        let address = container.pointer_address();

        tracking
            .referenced_values
            .get(&address)
            .expect("expected shared value to be tracked")
    }

    fn assert_top_level(
        tracking: &SharedValueTracking,
        container: &SharedContainer,
        expected_index: StackIndex,
    ) {
        match tracked_value(tracking, container) {
            TrackedReference::Root {
                container: tracked_container,
                index,
                ..
            } => {
                assert_eq!(*index, expected_index);
                assert_eq!(
                    tracked_container.pointer_address(),
                    container.pointer_address(),
                );
            }
            TrackedReference::Child { .. } => {
                panic!("expected top-level tracked value")
            }
            TrackedReference::_Uninitialized => unreachable!()
        }
    }

    fn assert_child(
        tracking: &SharedValueTracking,
        container: &SharedContainer,
    ) {
        match tracked_value(tracking, container) {
            TrackedReference::Child {
                container: tracked_container,
                ..
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

    fn tracking() -> SharedValueTracking<'static> {
        unsafe { default_tracking() }
    }

    #[test]
    fn index_start_at_one() {
        let tracking = tracking();
        assert_eq!(tracking.referenced_values.len(), 0);
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
        assert_eq!(tracking.referenced_values.len(), 1);

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
        assert_eq!(tracking.referenced_values.len(), 1);

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
        assert_eq!(tracking.referenced_values.len(), 2);

        assert_top_level(&tracking, &first, StackIndex(0));
        assert_top_level(&tracking, &second, StackIndex(1));
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

        let mut tracking = tracking();
        let parent_index = tracking.register_shared_value(parent.clone());

        assert_eq!(parent_index, StackIndex(0));
        assert_eq!(tracking.referenced_values.len(), 2);

        assert_top_level(&tracking, &parent, StackIndex(0));
        assert_child(&tracking, &child);
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
        assert_eq!(tracking.referenced_values.len(), 2);

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
        assert_eq!(tracking.referenced_values.len(), 3);

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
        assert_eq!(tracking.referenced_values.len(), 1);

        assert_top_level(&tracking, &parent, StackIndex(0));
    }
}
