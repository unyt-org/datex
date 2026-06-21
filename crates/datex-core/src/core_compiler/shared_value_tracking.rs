use crate::{
    collections::HashMap,
    global::protocol_structures::instruction_data::StackIndex,
    prelude::*,
    shared_values::{
        OwnedSharedContainer, PointerAddress, SelfOwnedPointerAddress,
        SharedContainer,
    },
};
use core::cell::Ref;
use itertools::Itertools;
use crate::traits::child_iterator::ChildIterator;
use crate::values::value_container::ValueContainer;

#[derive(Debug)]
pub enum TrackedValue {
    TopLevel {
        container: SharedContainer,
        index: StackIndex,
    },
    Child {
        container: SharedContainer,
    }
}

impl TrackedValue {
    fn update_container(
        &mut self,
        container: SharedContainer,
    ) {
        match self {
            TrackedValue::TopLevel { container: existing, .. } => {
                if container.ownership() > existing.ownership() {
                    *existing = container;
                }
            },
            TrackedValue::Child { container: existing } => {
                if container.ownership() > existing.ownership() {
                    *existing = container;
                }
            }
        }
    }
}


/// Helper struct used during compilation to keep track which shared values are moved or referenced
#[derive(Debug)]
pub struct SharedValueTracking {
    /// shared values that were injected in the compiler, with a reference mutability if referenced, or None if moved
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
        self.current_stack_index =
            StackIndex(self.current_stack_index.0 + 1);
        stack_index
    }

    /// Registers a new shared value. Returns a stack index that can be used to access this value
    pub fn register_shared_value(
        &mut self,
        shared_container: SharedContainer,
    ) -> StackIndex {
        let address = shared_container.pointer_address();
        self.register_shared_value_with_parents(shared_container, &HashSet::new());

        let tracked_value = self.shared_values.get(&address).unwrap();

        // ensure tracked value is a top level tracked value with stack index
        match tracked_value {
            TrackedValue::Child {..} => {
                let index = self.get_next_stack_index();
                match self.shared_values.remove(&address) {
                    Some(TrackedValue::Child { container }) => {
                        self.shared_values.insert(address, TrackedValue::TopLevel { container, index });
                    }
                    _ => unreachable!()
                }
                index
            },
            // already a top level value, do nothing
            TrackedValue::TopLevel { index, .. } => *index,
        }
    }

    fn register_shared_value_with_parents(
        &mut self,
        shared_container: SharedContainer,
        parents: &HashSet<SharedContainer>,
    )  {
        // register children recursively
        let parent = shared_container.clone();
        shared_container.value_container().with_collapsed_value(|value| {
            for child in value.iter_children() {
                match child {
                    ValueContainer::Shared(child) => self.register_shared_value_with_parents(
                        child.clone(),
                        &parents.clone().into_iter().chain(core::iter::once(parent.clone())).collect(),
                    ),
                    _ => {}
                }
            }
        });

        let address = shared_container.pointer_address();
        if let Some(tracked_value) = self.shared_values.get_mut(&address)
        {
            // replace if new container has higher ownership level than existing
            tracked_value.update_container(shared_container);
        } else {
            self.shared_values
                .insert(address.clone(), TrackedValue::Child {container: shared_container});
        }
    }

    /// Extracts all registered owned and referenced shared values
    pub fn into_tracked_values(self) -> Vec<TrackedValue> {
        self.shared_values
            .into_values()
            .collect()
    }
}
