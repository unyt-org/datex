use crate::collections::HashMap;

use crate::{
    prelude::*, shared_values::PointerAddress,
    values::core_values::endpoint::Endpoint,
};

#[derive(Debug, Clone, Default)]
pub struct PointerAvailabilityLookup {
    local_endpoint: Endpoint,
    lookup: HashMap<Endpoint, Vec<PointerAddress>>,
}

impl PointerAvailabilityLookup {
    pub fn new(origin: Endpoint) -> Self {
        Self {
            local_endpoint: origin,
            lookup: HashMap::new(),
        }
    }
    pub fn is_available_for_endpoint(
        &self,
        endpoint: &Endpoint,
        pointer_address: &PointerAddress,
    ) -> bool {
        if endpoint.is_local_or_equals_endpoint(&self.local_endpoint)
            && matches!(pointer_address, PointerAddress::SelfOwned(_))
        {
            return true;
        }
        false // TODO
    }
    pub fn is_available_for_all_endpoints(
        &self,
        endpoints: &[Endpoint],
        pointer_address: &PointerAddress,
    ) -> bool {
        for endpoint in endpoints {
            if !self.is_available_for_endpoint(endpoint, pointer_address) {
                return false;
            }
        }
        true
    }
}
