use crate::collections::HashMap;
use crate::global::protocol_structures::instruction_data::SlotAddress;
use crate::shared_values::pointer::PointerReferenceMutability;
use crate::shared_values::shared_container::{SharedContainerInner};

/// Helper struct used during compilation to keep track which shared values are moved or referenced
#[derive(Debug)]
pub struct SharedValueTracking {
    /// shared values that were injected in the compiler, with a reference mutability if referenced, or None if moved
    pub shared_values: HashMap<SharedContainerInner, (Option<PointerReferenceMutability>, SlotAddress)>,
    pub current_slot_address: SlotAddress
}

impl SharedValueTracking {

    pub fn new(start_address: SlotAddress) -> SharedValueTracking {
        SharedValueTracking {
            shared_values: HashMap::new(),
            current_slot_address: start_address,
        }
    }

    /// Registers a new shared value with minimum required ownership. Returns a slot address that can be used to access this value
    pub fn register_shared_value(&mut self, shared_value: SharedContainerInner, ownership: Option<PointerReferenceMutability>) -> SlotAddress {
        if let Some((existing, address)) = self.shared_values.get(&shared_value) {
            let address = *address;
            self.shared_values.insert(shared_value, (Self::max_ownership(existing, &ownership), address));
            address
        } else {
            let address = self.current_slot_address;
            self.current_slot_address = SlotAddress(self.current_slot_address.0 + 1);
            self.shared_values.insert(shared_value, (ownership, address));
            address
        }
    }

    /// Determines the maximum required ownership for a shared value based on the current tracking and a new reference mutability.
    fn max_ownership(current: &Option<PointerReferenceMutability>, new: &Option<PointerReferenceMutability>) -> Option<PointerReferenceMutability> {
        // both the same, no change
        if current == new {
            *current
        }
        // at least one move -> move required
        else if current.is_none() || new.is_none() {
            None
        }
        // at least one mutable -> mutable reference
        else if current.unwrap() == PointerReferenceMutability::Mutable || new.unwrap() == PointerReferenceMutability::Mutable {
            Some(PointerReferenceMutability::Mutable)
        }
        // default: immutable reference
        else {
            Some(PointerReferenceMutability::Immutable)
        }
    }
}