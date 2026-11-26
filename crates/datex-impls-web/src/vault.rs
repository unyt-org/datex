use datex_facades::vault::Vault;
use strum_macros::EnumString;

#[derive(Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum WebStorageLocation {
    LocalStorage,
    SessionStorage,
}

pub struct VaultWeb {
    location: WebStorageLocation
}

impl Vault for VaultWeb {
    fn new(location: &str) -> Self {
        let location = location.parse().expect("invalid storage location, must be one of 'local_storage' or 'session_storage'");
        VaultWeb { location }
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