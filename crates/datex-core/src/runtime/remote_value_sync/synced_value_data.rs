use crate::collections::HashMap;

use crate::{
    runtime::execution::context::RemoteExecutionContext,
    shared_values::{
        PointerAddress, SharedContainer, Subscribers,
        base_shared_value_container::observers::ObserverId,
    },
};
use core::assert_matches;

#[derive(Debug, Default)]
pub struct SyncedValueData {
    subscribers: HashMap<SharedContainer, Subscribers>,
    owner_observers: HashMap<SharedContainer, ObserverId>,
}

impl SyncedValueData {
    pub fn get_subscribers_mut(
        &mut self,
        shared: &SharedContainer,
    ) -> Option<&mut Subscribers> {
        self.subscribers.get_mut(shared)
    }
    pub fn get_subscribers(
        &self,
        shared: &SharedContainer,
    ) -> Option<&Subscribers> {
        self.subscribers.get(shared)
    }

    pub fn remote_execution_context(
        &self,
        container: &SharedContainer,
    ) -> Option<&RemoteExecutionContext> {
        self.subscribers
            .get(container)
            .map(|subscribers| subscribers.remote_execution_context())
    }

    pub fn remote_execution_context_mut(
        &mut self,
        container: &SharedContainer,
    ) -> Option<&mut RemoteExecutionContext> {
        self.subscribers
            .get_mut(container)
            .map(|subscribers| subscribers.remote_execution_context_mut())
    }

    /// Registers a shared container.
    /// # Safety
    /// The caller must ensure, that the shared container has a owned address
    pub unsafe fn set_subscribers(
        &mut self,
        shared: &SharedContainer,
        subscribers: Subscribers,
    ) -> &mut Subscribers {
        assert_matches!(
            shared.pointer_address(),
            PointerAddress::SelfOwned(_),
            "Shared container must have a self-owned address"
        );
        self.subscribers.insert(shared.clone(), subscribers);
        self.subscribers.get_mut(shared).unwrap()
    }

    pub fn get_owner_observer(
        &self,
        shared: &SharedContainer,
    ) -> Option<&ObserverId> {
        self.owner_observers.get(shared)
    }

    pub fn delete_owner_observer(&mut self, shared: &SharedContainer) {
        self.owner_observers.remove(shared);
    }

    pub fn set_owner_observer(
        &mut self,
        shared: SharedContainer,
        observer: ObserverId,
    ) {
        self.owner_observers.insert(shared, observer);
    }
}
