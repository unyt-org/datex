use crate::collections::HashMap;

use crate::{
    prelude::*,
    runtime::execution::context::RemoteExecutionContext,
    shared_values::{
        ReferenceMutability, base_shared_value_container::observers::ObserverId,
    },
    values::core_values::endpoint::Endpoint,
};

#[derive(Debug)]
pub struct Subscribers {
    lookup: HashMap<Endpoint, SubscriptionMetadata>,
    observer_id: Option<ObserverId>,
    remote_execution_context: RemoteExecutionContext,
}

impl Subscribers {
    pub fn new(observer_id: Option<ObserverId>, remote_execution_context: RemoteExecutionContext) -> Self {
        Self {
            lookup: HashMap::new(),
            observer_id,
            remote_execution_context,
        }
    }

    /// Adds a subscriber to the list of subscribers.
    /// If the subscriber already exists, the access rights are updated.
    pub fn add_subscriber(
        &mut self,
        endpoint: &Endpoint,
        access_rights: ReferenceMutability,
    ) {
        // FIXME endpoint instance handling
        self.lookup.insert(
            endpoint.clone(),
            SubscriptionMetadata { access_rights }, // TODO merge if other properties relevant
        );
    }
    pub fn observer_id(&self) -> Option<ObserverId> {
        self.observer_id
    }

    pub fn remove_subscriber(&mut self, endpoint: &Endpoint) {
        self.lookup.remove(&endpoint.any_instance());
    }

    pub fn subscriber_metadata(
        &self,
        endpoint: &Endpoint,
    ) -> Option<&SubscriptionMetadata> {
        self.lookup.get(&endpoint.any_instance())
    }

    pub fn lookup(&self) -> &HashMap<Endpoint, SubscriptionMetadata> {
        &self.lookup
    }

    pub fn endpoints(&self) -> Vec<Endpoint> {
        self.lookup.keys().cloned().collect()
    }
    
    pub fn remote_execution_context(&self) -> &RemoteExecutionContext {
        &self.remote_execution_context
    }

    pub fn remote_execution_context_mut(&mut self) -> &mut RemoteExecutionContext {
        &mut self.remote_execution_context
    }
}

#[derive(Debug, PartialEq)]
pub struct SubscriptionMetadata {
    pub access_rights: ReferenceMutability,
}
