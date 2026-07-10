pub mod owned_shared_subscriptions;

use alloc::rc::Rc;
use core::ops::Deref;
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
use crate::core_compiler::core_compilation_context::CompileInput;
use crate::core_compiler::update_compiler::compile_updates;
use crate::runtime::execution::context::{ExecutionMode, RemoteExecutionContext};
use crate::prelude::*;

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
                me.handle_update(&container, data)
            }))?;
            unsafe {
                owned_pointer_subscriptions
                    .set_subscribers(shared_container, Subscribers::new(id))
            }
        };
        subscribers.add_subscriber(endpoint, access_rights);

        Ok(())
    }

    fn handle_update(self: &Rc<Self>, container: &SharedContainer, update: &Update) {
        let subscriber = self.owned_pointer_subscriptions();
        let subscribers = subscriber.get_subscribers(container);
        if let Some(subscribers) = subscribers {
            let endpoints = subscribers.endpoints();
            self.task_manager().register_task(
                // TODO: no clone?
                self.clone().send_update_block(container.clone(), update.clone(), endpoints)
            );
        }
    }

    async fn send_update_block(
        self: Rc<RuntimeInternal>,
        container: SharedContainer,
        update: Update,
        receiver_endpoints: Vec<Endpoint>,
    ) {
        let update_dxb = {
            let lookup = self.pointer_availability_lookup();
            let input = CompileInput::new(
                lookup.deref(),
                &receiver_endpoints
            );

            compile_updates(container, &[&update.data], input)
        };

        // TODO: receiver_endpoints
        let mut context = RemoteExecutionContext::new(
            self.endpoint().clone(),
            ExecutionMode::Static,
            self.clone().into(),
        );

        self.execute_remote(
            &mut context,
            update_dxb
        ).await.expect("Failed to execute remote update block");
    }
}
