use alloc::string::String;
use datex_facades::vault::Vault;

pub struct VaultEsp;

impl Vault for VaultEsp {
    fn new(location: &str) -> Self {
        VaultEsp
    }
    fn store(&self, endpoint: &str, key: &str, data: &str) {
        todo!()
    }

    fn get(&self, endpoint: &str, key: &str) -> Option<String> {
        todo!()
    }

    fn delete(&self, endpoint: &str, key: &str) {
        todo!()
    }
}