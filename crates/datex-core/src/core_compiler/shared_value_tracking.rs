use core::cell::{Ref, RefCell};
use crate::collections::HashMap;
use crate::global::protocol_structures::instruction_data::StackIndex;
use crate::shared_values::pointer_address::{SelfOwnedPointerAddress, PointerAddress};
use crate::shared_values::shared_containers::{OwnedSharedContainer, ReferenceMutability, SharedContainer};

/// Helper struct used during compilation to keep track which shared values are moved or referenced
#[derive(Debug)]
pub struct SharedValueTracking {
    /// shared values that were injected in the compiler, with a reference mutability if referenced, or None if moved
    pub shared_values: HashMap<PointerAddress, (SharedContainer, StackIndex)>,
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
    pub fn register_shared_value(&mut self, shared_container: SharedContainer) -> StackIndex {
        let address = shared_container.pointer_address();
        if let Some((existing, stack_index)) = self.shared_values.get(&address) {
            let stack_index = *stack_index;
            // new container has higher ownership level than existing
            if shared_container.ownership() > existing.ownership() {
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

    /// Extracts all registered owned shared values
    pub fn into_moved_shared_values(self) -> Vec<OwnedSharedContainer> {
        self.shared_values
            .into_iter()
            .filter_map(|(_, (container, _))| {
                match container {
                    SharedContainer::Owned(owned) => Some(owned),
                    SharedContainer::Referenced(_) => None,
                }
            })
            .collect()
    }

    /// Get all registered owned shared values
    pub fn get_moved_shared_addresses(&self) -> Vec<Ref<SelfOwnedPointerAddress>> {
        self.shared_values
            .iter()
            .filter_map(|(_, (container, _))| {
                match container {
                    SharedContainer::Owned(owned) => Some(owned.pointer_address()),
                    SharedContainer::Referenced(_) => None,
                }
            })
            .collect()
    }
}