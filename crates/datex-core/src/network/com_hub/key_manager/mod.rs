pub mod own_endpoint_keys;

use core::cell::RefCell;
use core::cell::Ref;
use crate::collections::HashMap;
use crate::network::com_hub::key_manager::own_endpoint_keys::OwnEndpointKeys;
use crate::values::core_values::endpoint::Endpoint;


pub struct KeyManager {
    /// The endpoint's own keys
    own_keys: RefCell<OwnEndpointKeys>,
    /// A map of peer endpoints to their public keys, used for signature validation of incoming blocks.
    endpoint_keys: RefCell<HashMap<Endpoint, [u8; 32]>>,
}


impl KeyManager {
    pub fn new(own_keys: OwnEndpointKeys) -> Self {
        KeyManager {
            own_keys: RefCell::new(own_keys),
            endpoint_keys: RefCell::new(HashMap::new()),
        }
    }
    
    /// Overwrites the endpoint's own keys with the provided `OwnEndpointKeys`.
    pub fn set_own_keys(&self, own_keys: OwnEndpointKeys) {
        *self.own_keys.borrow_mut() = own_keys;
    }

    pub fn own_keys(&self) -> Ref<'_, OwnEndpointKeys> {
        self.own_keys.borrow()
    }

    pub fn endpoint_keys(&self) -> Ref<'_, HashMap<Endpoint, [u8; 32]>> {
        self.endpoint_keys.borrow()
    }

    pub fn get_endpoint_key(&self, endpoint: &Endpoint) -> Option<[u8; 32]> {
        self.endpoint_keys.borrow().get(endpoint).copied()
    }

    /// Registers the public key of a peer endpoint in the peer vault for signature validation of incoming blocks.
    /// If the endpoint is already registered, returns an error.
    pub fn register_endpoint_key(
        &self,
        endpoint: Endpoint,
        pub_key: [u8; 32],
    ) -> Result<(), ()> {
        let mut peer_vault = self.endpoint_keys.borrow_mut();
        if peer_vault.contains_key(&endpoint) {
            // check if the existing key is the same as the new key, if not return error
            return if peer_vault.get(&endpoint) != Some(&pub_key) {
                Err(())
            } else {
                Ok(())
            };
        }
        let _ = peer_vault.insert(endpoint, pub_key);
        Ok(())
    }

    pub fn pub_key(&self) -> Ref<'_, [u8; 32]> {
        Ref::map(self.own_keys(), |vault| &vault.pub_key)
    }
}