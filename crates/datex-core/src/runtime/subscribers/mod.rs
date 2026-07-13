pub mod owned_shared_subscriptions;

use crate::{
    core_compiler::{
        core_compilation_context::CompileInput,
        update_compiler::compile_updates,
    },
    prelude::*,
    runtime::{
        RuntimeInternal,
        execution::context::{ExecutionMode, RemoteExecutionContext},
    },
    shared_values::{
        PointerAddress, ReferenceMutability, SharedContainer,
        SharedContainerMutability, Subscribers,
        base_shared_value_container::observers::{
            Observer, ObserverError, ObserverId,
        },
        traits::SharedContainerCommon,
    },
    value_updates::update_data::Update,
    values::core_values::endpoint::Endpoint,
};
use alloc::rc::Rc;
use core::{assert_matches, fmt::Display, ops::Deref};

#[derive(Debug)]
pub enum SubscriberError {
    NotASelfOwnedContainer,
    Observer(ObserverError),
}

impl From<ObserverError> for SubscriberError {
    fn from(err: ObserverError) -> Self {
        SubscriberError::Observer(err)
    }
}

impl Display for SubscriberError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SubscriberError::NotASelfOwnedContainer => {
                write!(f, "The shared container is not self-owned.")
            }
            SubscriberError::Observer(err) => {
                write!(f, "Observer error: {}", err)
            }
        }
    }
}

impl RuntimeInternal {
    /// Subscribes an endpoint to a shared container with the specified access rights.
    pub fn subscribe_endpoint(
        self: Rc<Self>,
        shared_container: &SharedContainer,
        endpoint: &Endpoint,
        access_rights: ReferenceMutability,
    ) -> Result<(), SubscriberError> {
        if !matches!(
            shared_container.pointer_address(),
            PointerAddress::SelfOwned(_)
        ) {
            return Err(SubscriberError::NotASelfOwnedContainer);
        }

        let mut owned_pointer_subscriptions =
            self.owned_pointer_subscriptions_mut();
        let subscribers = if let Some(subscribers) =
            owned_pointer_subscriptions.get_subscribers_mut(shared_container)
        {
            subscribers
                .remote_execution_context_mut()
                .add_endpoint(endpoint.clone());
            // TODO: when new subscriber is added, send initial block for context
            subscribers
        } else {
            let observer_id =
                if shared_container.base_shared_container().is_mutable() {
                    let container = shared_container.clone();
                    let me = self.clone();
                    Some(shared_container.observe(Observer::new(
                        move |data| me.handle_update(&container, data),
                    ))?)
                } else {
                    None
                };

            let context = RemoteExecutionContext::new(
                vec![self.endpoint().clone()],
                ExecutionMode::Static,
                self.clone().into(),
            );

            unsafe {
                owned_pointer_subscriptions.set_subscribers(
                    shared_container,
                    Subscribers::new(observer_id, context),
                )
            }
        };
        subscribers.add_subscriber(endpoint, access_rights);

        Ok(())
    }

    /// Handles an update to a shared container by notifying all subscribed endpoints.
    fn handle_update(
        self: &Rc<Self>,
        container: &SharedContainer,
        update: &Update,
    ) {
        let subscriber = self.owned_pointer_subscriptions();
        let subscribers = subscriber.get_subscribers(container);
        if let Some(subscribers) = subscribers {
            let endpoints = subscribers.endpoints();
            self.task_manager()
                .register_task(self.clone().send_update_block(
                    // TODO: no clone?
                    container.clone(),
                    update.clone(),
                    endpoints,
                ));
        }
    }

    /// Compiles a DXB block for the given update and sends it to the specified receiver endpoints.
    /// Note: this function asserts that the shared container is still owned and that the remote execution
    /// context still exists.
    async fn send_update_block(
        self: Rc<RuntimeInternal>,
        container: SharedContainer,
        update: Update,
        receiver_endpoints: Vec<Endpoint>,
    ) {
        assert_matches!(
            container.pointer_address(),
            PointerAddress::SelfOwned(_)
        );

        let update_dxb = {
            let lookup = self.pointer_availability_lookup();
            let input = CompileInput::new(lookup.deref(), &receiver_endpoints);

            compile_updates(&container, &[&update.data], input)
        };

        let self_clone = self.clone();

        let subscriptions = self.owned_pointer_subscriptions();
        let context =
            subscriptions.remote_execution_context(&container).unwrap();

        println!(
            "Sending update block to endpoints: {:?}",
            receiver_endpoints
        );

        self_clone
            .execute_remote(context, update_dxb)
            .await
            .expect("Failed to execute remote update block");
    }
}
