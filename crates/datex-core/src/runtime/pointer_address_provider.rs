use crate::{
    global::protocol_structures::instruction_data::RawRemotePointerAddress,
    prelude::*,
    shared_values::{
        ExternalPointerAddress, PointerAddress, SelfOwnedPointerAddress,
    },
    values::core_values::endpoint::Endpoint,
};
use binrw::io::Cursor;

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
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            local_endpoint: endpoint,
            local_counter: 0,
            last_timestamp: 0,
        }
    }

    /// Takes a RawFullPointerAddress and converts it to a PointerAddress::Local or PointerAddress::Remote,
    /// depending on whether the pointer origin id matches the local endpoint.
    pub fn get_pointer_address_from_raw_full_address(
        &self,
        raw_address: RawRemotePointerAddress,
    ) -> PointerAddress {
        if let Ok(endpoint) = raw_address.endpoint()
            && endpoint == self.local_endpoint
        {
            // TODO #639: check if it makes sense to take the last 5 bytes only here
            let last_bytes = &raw_address.id[raw_address.id.len() - 5..];
            PointerAddress::self_owned(last_bytes.try_into().unwrap())
        } else {
            // combine raw_address.endpoint and raw_address.id to [u8; 26]
            let writer = Cursor::new(Vec::new());
            let mut bytes = writer.into_inner();
            bytes.extend_from_slice(&raw_address.id);
            PointerAddress::External(ExternalPointerAddress::Remote(
                <[u8; 26]>::try_from(bytes).unwrap(),
            ))
        }
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
