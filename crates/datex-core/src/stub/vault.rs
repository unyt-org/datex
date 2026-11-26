use datex_facades::vault::Vault;

pub struct VaultStub;

impl Vault for VaultStub {
    fn store(&self, endpoint: &str, key: &str, data: &[u8]) {
        unimplemented!()
    }

    fn get(&self, endpoint: &str, key: &str) -> Option<Vec<u8>> {
        unimplemented!()
    }

    fn delete(&self, endpoint: &str, key: &str) {
        unimplemented!()
    }
}