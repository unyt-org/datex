use crate::{
    collections::HashMap,
    global::protocol_structures::instruction_data::RawRemotePointerAddress,
    libs::core::{CoreLibTypeId, core_lib_id::CoreLibId, load_core_lib},
    prelude::*,
    shared_values::{
        pointer_address::{
            ExternalPointerAddress, PointerAddress, SelfOwnedPointerAddress,
        },
        shared_containers::{
            ReferencedSharedContainer, SharedContainer, SharedContainerInner,
            base_shared_value_container::BaseSharedValueContainer,
        },
    },
    types::error::IllegalTypeError,
    values::core_values::endpoint::Endpoint,
};
use binrw::io::Cursor;
use core::{cell::Ref, result::Result};
use std::cell::RefCell;

#[derive(Debug, Default)]
pub struct Memory {
    local_endpoint: Endpoint,
    /// Counter for local pointer ids
    local_counter: u64,
    /// Last timestamp used for a new local pointer id
    last_timestamp: u64,
    /// All non-local pointers
    pointers: HashMap<PointerAddress, ReferencedSharedContainer>,
}

impl Memory {
    /// Creates a new, Memory instance with the core library loaded.
    pub fn new(endpoint: Endpoint) -> Memory {
        let mut memory = Memory {
            local_endpoint: endpoint,
            local_counter: 0,
            last_timestamp: 0,
            pointers: HashMap::new(),
        };
        // load core library
        load_core_lib(&mut memory);
        memory
    }

    /// Registers a referenced shared container in memory. If the reference has no PointerAddress, a new local one is generated.
    /// If the reference is already registered (has a PointerAddress), the existing address is returned and no new registration is done.
    /// Owned shared containers shall not be registered in memory.
    /// Returns the PointerAddress of the registered reference.
    pub fn register_referenced_shared_container(
        &mut self,
        container: &ReferencedSharedContainer,
    ) {
        let pointer_address = container.pointer_address();
        // check if reference is already registered (if it has an address, we assume it is registered)
        self.pointers
            .entry(pointer_address)
            .or_insert_with(|| container.clone());
    }

    /// Returns a reference stored at the given PointerAddress, if it exists.
    pub fn get_reference(
        &self,
        pointer_address: &PointerAddress,
    ) -> Option<&ReferencedSharedContainer> {
        self.pointers.get(pointer_address)
    }

    /// Helper function to get a core value directly from memory
    pub fn get_core_reference(
        &self,
        pointer_id: CoreLibId,
    ) -> &ReferencedSharedContainer {
        self.get_reference(&pointer_id.into())
            .expect("core reference not found in memory")
    }

    /// Helper function to get a core type directly from memory if it can be used as a type
    pub fn get_core_type_reference(
        &self,
        pointer_id: CoreLibTypeId,
    ) -> Result<Ref<SharedTypeContainer>, IllegalTypeError> {
        let reference = self
            .get_reference(&pointer_id.into())
            .ok_or(IllegalTypeError::TypeNotFound)?;

        Ref::filter_map(reference.value(), |container| match container {
            SharedContainerValueOrType::Type(def) => Some(def),
            _ => None,
        })
        .map_err(|_| IllegalTypeError::TypeNotFound)
    }

    /// Helper function to get a core type directly from memory, asserting that is can be used as a type
    /// Panics if the core type is not found or cannot be used as a type.
    pub fn get_core_type_reference_unchecked(
        &self,
        pointer_id: CoreLibTypeId,
    ) -> Ref<SharedTypeContainer> {
        // FIXME #415: Mark as unchecked
        self.get_core_type_reference(pointer_id)
            .expect("core type not found or cannot be used as a type")
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
            PointerAddress::owned(last_bytes.try_into().unwrap())
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

    /// Creates a new unique local owned pointer.
    pub fn get_new_endpoint_owned_pointer_address(
        &mut self,
    ) -> SelfOwnedPointerAddress {
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
