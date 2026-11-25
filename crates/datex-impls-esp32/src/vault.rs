use alloc::vec::Vec;
use datex_facades::vault::Vault;

pub struct VaultEsp;

impl Vault for VaultEsp {
    fn store(&self, endpoint: &str, key: &str, data: &[u8]) {
        todo!()
    }

    fn get(&self, endpoint: &str, key: &str) -> Option<Vec<u8>> {
        todo!()
    }

    fn delete(&self, endpoint: &str, key: &str) {
        todo!()
    }
}