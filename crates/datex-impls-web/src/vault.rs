use datex_facades::vault::Vault;

pub struct VaultWeb;

impl Vault for VaultWeb {
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