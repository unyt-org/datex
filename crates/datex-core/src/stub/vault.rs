use datex_facades::vault::Vault;
use crate::prelude::*;

pub struct VaultStub;

impl Vault for VaultStub {
    fn new(_location: &str) -> Self {
        VaultStub
    }
    /// Has no effect in the stub implementation
    fn store(&self, _endpoint: &str, _key: &str, _data: &str) {}

    /// Stub implementation always returns None
    fn get(&self, _endpoint: &str, _key: &str) -> Option<String> {
        None
    }

    /// Has no effect in the stub implementation
    fn delete(&self, _endpoint: &str, _key: &str) {}
}