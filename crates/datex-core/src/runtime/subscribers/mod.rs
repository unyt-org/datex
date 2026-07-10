pub mod owned_shared_subscriptions;

use alloc::rc::Rc;

use crate::{
    runtime::RuntimeInternal,
    shared_values::{
        ReferenceMutability, SharedContainer, Subscribers,
        base_shared_value_container::observers::{
            Observer, ObserverError, ObserverId,
        },
    },
    value_updates::update_data::Update,
    values::core_values::endpoint::Endpoint,
};

impl RuntimeInternal {
    /// Subscribes an endpoint to a shared container with the specified access rights.
    /// # Safety
    /// The caller must ensure that the shared container has a self-owned address.
    pub unsafe fn subscribe_endpoint(
        self: Rc<Self>,
        shared_container: &SharedContainer,
        endpoint: Endpoint,
        access_rights: ReferenceMutability,
    ) -> Result<(), ObserverError> {
        let mut owned_pointer_subscriptions =
            self.owned_pointer_subscriptions_mut();
        let subscribers = if let Some(subscriber) = unsafe {
            owned_pointer_subscriptions.get_subscribers_mut(shared_container)
        } {
            subscriber
        } else {
            let container = shared_container.clone();
            let me = self.clone();
            let id = shared_container.observe(Observer::new(move |data| {
                me.observe(&container, data);
            }))?;
            unsafe {
                owned_pointer_subscriptions
                    .set_subscribers(shared_container, Subscribers::new(id))
            }
        };
        subscribers.add_subscriber(endpoint, access_rights);

        Ok(())
    }

    fn observe(self: &Rc<Self>, container: &SharedContainer, data: &Update) {
        let subscriber = self.owned_pointer_subscriptions();
        let subscribers = subscriber.get_subscribers(container);
        if let Some(subscribers) = subscribers {
            let endpoints = subscribers.endpoints();
        }
    }
}
