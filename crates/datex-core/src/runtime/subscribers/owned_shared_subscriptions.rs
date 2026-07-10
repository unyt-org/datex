use crate::collections::HashMap;

use crate::shared_values::{PointerAddress, SharedContainer, Subscribers};
use core::assert_matches;
#[derive(Debug, Default)]
pub struct OwnedSharedSubscriptions {
    subscriptions: HashMap<SharedContainer, Subscribers>,
}

impl OwnedSharedSubscriptions {
    pub fn get_subscribers_mut(
        &mut self,
        shared: &SharedContainer,
    ) -> Option<&mut Subscribers> {
        self.subscriptions.get_mut(shared)
    }
    pub fn get_subscribers(
        &self,
        shared: &SharedContainer,
    ) -> Option<&Subscribers> {
        self.subscriptions.get(shared)
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
        self.subscriptions.insert(shared.clone(), subscribers);
        self.subscriptions.get_mut(shared).unwrap()
    }
}
