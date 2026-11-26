use core::cell::RefCell;
use core::cell::Ref;
use log::{info, warn};
use crate::collections::HashMap;
use crate::crypto::CryptoImpl;
use datex_facades::crypto::Crypto;
use crate::values::core_values::endpoint::Endpoint;

/// Contains the endpoint's own public/private key pair
pub struct OwnEndpointKeys {
    pub pub_key: [u8; 32],
    pub pri_key: [u8; 32],
}


#[derive(Default)]
pub struct KeyManager {
    /// The endpoint's own keys, is guaranteed to exist on an initialized ComHub after calling [Self::init_own_keys]
    own_keys: RefCell<Option<OwnEndpointKeys>>,
    /// A map of peer endpoints to their public keys, used for signature validation of incoming blocks.
    endpoint_keys: RefCell<HashMap<Endpoint, [u8; 32]>>,
 }

impl KeyManager {
    /// Stores the provided keys in the vault or generates new keys if none are provided.
    pub async fn init_own_keys(
        &self,
        pub_key_str: Option<&str>,
        pri_key_str: Option<&str>,
    ) {
        if self.own_keys.borrow().is_some() {
            core::panic!("ComHub keys already initialized");
        }

        let (pub_key, pri_key) = match (pub_key_str, pri_key_str) {
            (Some(pub_key_str), Some(pri_key_str)) => (
                CryptoImpl::dec_b58_32(pub_key_str).unwrap(),
                CryptoImpl::dec_b58_32(pri_key_str).unwrap(),
            ),
            _ => {
                let (pub_key, pri_key) =
                    CryptoImpl::gen_ed25519().await.unwrap();
                (pub_key, pri_key)
            }
        };

        info!("Initializing com hub pub key: {:?}", pub_key);

        let mut own_keys = self.own_keys.borrow_mut();
        *own_keys = Some(OwnEndpointKeys { pub_key, pri_key });
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