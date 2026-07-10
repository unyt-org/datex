use crate::collections::HashMap;

use crate::{
    shared_values::{
        ReferenceMutability, base_shared_value_container::observers::ObserverId,
    },
    values::core_values::endpoint::Endpoint,
};

#[derive(Debug)]
pub struct Subscribers {
    lookup: HashMap<Endpoint, SubscriptionMetadata>,
    observer_id: ObserverId,
}

impl Subscribers {
    pub fn new(observer_id: ObserverId) -> Self {
        Self {
            lookup: HashMap::new(),
            observer_id,
        }
    }

    /// Adds a subscriber to the list of subscribers.
    /// If the subscriber already exists, the access rights are updated.
    pub fn add_subscriber(
        &mut self,
        endpoint: Endpoint,
        access_rights: ReferenceMutability,
    ) {
        // FIXME endpoint instance handling
        self.lookup.insert(
            endpoint.any_instance(),
            SubscriptionMetadata { access_rights }, // TODO merge if other properties relevant
        );
    }
    pub fn observer_id(&self) -> ObserverId {
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
}

#[derive(Debug, PartialEq)]
pub struct SubscriptionMetadata {
    pub access_rights: ReferenceMutability,
}
