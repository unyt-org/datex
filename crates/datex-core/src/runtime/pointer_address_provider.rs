use crate::{
    prelude::*,
    shared_values::{
        PointerAddress, RemotePointerAddress, SelfOwnedPointerAddress,
    },
    values::core_values::endpoint::Endpoint,
};

#[derive(Default, Debug)]
pub struct SelfOwnedPointerAddressProvider {
    local_endpoint: Endpoint,
    /// Counter for local pointer ids
    local_counter: u64,
    /// Last timestamp used for a new local pointer id
    last_timestamp: u64,
}

impl SelfOwnedPointerAddressProvider {
    /// Creates a new, [SelfOwnedPointerAddressProvider] instance
    pub const fn new(endpoint: Endpoint) -> Self {
        Self {
            local_endpoint: endpoint,
            local_counter: 0,
            last_timestamp: 0,
        }
    }

    /// Takes a [RemotePointerAddress] and converts it to a [PointerAddress::Local] or [PointerAddress::Remote],
    /// depending on whether the pointer origin id matches the local endpoint.
    pub fn normalize_address(
        &self,
        raw_address: RemotePointerAddress,
    ) -> PointerAddress {
        raw_address.normalize_for_local(&self.local_endpoint)
    }

    pub fn get_new_self_owned_address(&mut self) -> SelfOwnedPointerAddress {
        let timestamp = crate::time::now_ms();
        // new timestamp, reset counter
        if timestamp != self.last_timestamp {
            self.last_timestamp = timestamp;
            self.local_counter = 0;
        }
        // same timestamp as last time, increment counter to prevent collision
        else {
            self.local_counter += 1;
        }
        self.local_counter += 1;

        // create id: 4 bytes timestamp + 1 byte counter
        let id: [u8; 5] = [
            (timestamp >> 24) as u8,
            (timestamp >> 16) as u8,
            (timestamp >> 8) as u8,
            timestamp as u8,
            (self.local_counter & 0xFF) as u8,
        ];

        SelfOwnedPointerAddress::new(id)
    }
}
