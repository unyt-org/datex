use datex_facades::vault::Vault;
use crate::prelude::*;

pub struct VaultStub;

impl Vault for VaultStub {
    fn new(location: &str) -> Self {
        VaultStub
    }
    /// Has no effect in the stub implementation
    fn store(&self, endpoint: &str, key: &str, data: &str) {}

    /// Stub implementation always returns None
    fn get(&self, endpoint: &str, key: &str) -> Option<String> {
        None
    }

    /// Has no effect in the stub implementation
    fn delete(&self, endpoint: &str, key: &str) {}
}