use alloc::rc::Rc;
use core::cell::{Ref, RefCell};
use crate::collections::HashMap;
use crate::global::protocol_structures::instruction_data::StackIndex;
use crate::shared_values::pointer_address::{EndpointOwnedPointerAddress, PointerAddress};
use crate::shared_values::shared_container::{OwnedSharedContainer, SharedContainerValueOrType, SharedContainerInner};

/// Helper struct used during compilation to keep track which shared values are moved or referenced
#[derive(Debug)]
pub struct SharedValueTracking {
    /// shared values that were injected in the compiler, with a reference mutability if referenced, or None if moved
    pub shared_values: HashMap<PointerAddress, (SharedContainerValueOrType, StackIndex)>,
    pub current_stack_index: StackIndex
}

impl SharedValueTracking {

    pub fn new() -> SharedValueTracking {
        SharedValueTracking {
            shared_values: HashMap::new(),
            current_stack_index: StackIndex(1),
        }
    }

    /// Registers a new shared value. Returns a stack index that can be used to access this value
    pub fn register_shared_value(&mut self, shared_container: SharedContainerValueOrType) -> StackIndex {
        let address = shared_container.pointer_address();
        if let Some((existing, stack_index)) = self.shared_values.get(&address) {
            let stack_index = *stack_index;
            if Self::has_higher_ownership(existing, &shared_container) {
                self.shared_values.insert(address, (shared_container, stack_index));
            }
            stack_index
        } else {
            let stack_index = self.current_stack_index;
            self.current_stack_index = StackIndex(self.current_stack_index.0 + 1);
            self.shared_values.insert(address, (shared_container, stack_index));
            stack_index
        }
    }

    /// Determine whether the new container has a higher ownership level than the current
    fn has_higher_ownership(current: &SharedContainerValueOrType, new: &SharedContainerValueOrType) -> bool {
        let current_mutability = &current.reference_mutability;
        let new_current_mutability = &new.reference_mutability;

        // both the same, no change
        if current_mutability == new_current_mutability {
            return false;
        }

        match (new_current_mutability, current_mutability) {
            // mutable > immutable
            (Some(ReferenceMutability::Mutable), Some(ReferenceMutability::Immutable)) => true,
            // move > immutable, move > mutable
            (None, Some(ReferenceMutability::Immutable | ReferenceMutability::Mutable)) => true,
            _ => false
        }
    }

    /// Extracts all registered owned shared values
    pub fn into_moved_shared_values(self) -> Vec<OwnedSharedContainer> {
        self.shared_values
            .into_iter()
            .filter_map(|(_, (container, _))| {
                match container {
                    SharedContainerValueOrType::Owned(owned) => Some(owned),
                    SharedContainerValueOrType::Referenced(_) => None,
                }
            })
            .collect()
    }

    /// Get all registered owned shared values
    pub fn get_moved_shared_addresses(&self) -> Vec<Ref<EndpointOwnedPointerAddress>> {
        self.shared_values
            .iter()
            .filter_map(|(_, (container, _))| {
                match container {
                    SharedContainerValueOrType::Owned(owned) => Some(owned.pointer_address()),
                    SharedContainerValueOrType::Referenced(_) => None,
                }
            })
            .collect()
    }
}