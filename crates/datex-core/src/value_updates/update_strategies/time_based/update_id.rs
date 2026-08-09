use core::cmp::Ordering;

use crate::values::core_values::endpoint::Endpoint;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UpdateId {
    pub timestamp: u64,
    pub endpoint: Endpoint,
}

impl Ord for UpdateId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then(self.endpoint.cmp(&other.endpoint))
    }
}

impl PartialOrd for UpdateId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
