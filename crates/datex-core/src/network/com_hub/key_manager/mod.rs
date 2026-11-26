pub mod own_endpoint_keys;

use core::cell::RefCell;
use core::cell::Ref;
use binrw::io::Cursor;
use binrw::{BinRead, BinWrite};
use log::{info, warn};
use crate::collections::HashMap;
use crate::crypto::CryptoImpl;
use datex_facades::crypto::Crypto;
use datex_facades::vault::Vault;
use crate::network::com_hub::key_manager::own_endpoint_keys::OwnEndpointKeys;
use crate::values::core_values::endpoint::Endpoint;
use crate::vault::VaultImpl;


pub struct KeyManager {
    vault: Option<VaultImpl>,
    /// The endpoint's own keys, is guaranteed to exist on an initialized ComHub after calling [Self::init_own_keys]
    own_keys: RefCell<Option<OwnEndpointKeys>>,
    /// A map of peer endpoints to their public keys, used for signature validation of incoming blocks.
    endpoint_keys: RefCell<HashMap<Endpoint, [u8; 32]>>,
}


impl KeyManager {
    pub fn new(vault_location: Option<&str>) -> Self {
        KeyManager {
            vault: vault_location.map(|location| VaultImpl::new(location)),
            own_keys: RefCell::new(None),
            endpoint_keys: RefCell::new(HashMap::new()),
        }
    }
    
    /// Stores the provided keys in the vault or generates new keys if none are provided.
    pub async fn init_own_keys(
        &self,
        endpoint: &Endpoint,
        keys: Option<OwnEndpointKeys>,
    ) {
        if self.own_keys.borrow().is_some() {
            core::panic!("ComHub keys already initialized");
        }

        let keys = match keys {
            Some(keys) => { 
                // update the vault with the provided keys
                if let Some(vault) = &self.vault {
                    let serialized_keys = keys.serialize_to_base58().expect("Failed to serialize own keys");
                    vault.store( & endpoint.to_string(), "own_keys", & serialized_keys);
                }
                keys
            },
            _ => {
                let stored_keys = self.vault.as_ref().and_then(|vault| vault.get(&endpoint.to_string(), "own_keys"));

                // If keys are stored in the vault, use them. Otherwise, generate new keys and store them in the vault.
                match stored_keys {
                    Some(stored_keys) => {
                        let keys: OwnEndpointKeys = OwnEndpointKeys::deserialize_from_base58(&stored_keys).expect("Failed to deserialize own keys");
                        keys
                    }
                    None => {
                        let (pub_key, pri_key) =
                            CryptoImpl::gen_ed25519().await.unwrap();
                        let serialized_keys = OwnEndpointKeys { pub_key, pri_key }
                            .serialize_to_base58()
                            .expect("Failed to serialize own keys");
                        if let Some(vault) = &self.vault {
                            vault.store(&endpoint.to_string(), "own_keys", &serialized_keys);
                        }
                        OwnEndpointKeys { pub_key, pri_key }
                    }
                }
            }
        };

        info!("Initializing com hub pub key: {:?}", keys.pub_key);

        let mut own_keys = self.own_keys.borrow_mut();
        *own_keys = Some(keys);
    }

    pub fn own_keys(&self) -> Ref<'_, OwnEndpointKeys> {
        Ref::map(self.own_keys.borrow(), |vault| {
            vault.as_ref().expect("Vault is not initialized")
        })
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
            if peer_vault.get(&endpoint) != Some(&pub_key) {
                // FIXME: error, only warning for now until keys can be stored persistently
                warn!(
                    "Endpoint {} is already registered with a different public key",
                    endpoint
                );
                // return Err(());
            } else {
                return Ok(());
            }
        }
        let _ = peer_vault.insert(endpoint, pub_key);
        Ok(())
    }

    pub fn pub_key(&self) -> Ref<'_, [u8; 32]> {
        Ref::map(self.own_keys(), |vault| &vault.pub_key)
    }
}