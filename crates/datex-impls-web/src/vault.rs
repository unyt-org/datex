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

impl VaultWeb {
    fn storage_key(&self, endpoint: &str, key: &str) -> String {
        format!("vault:{}:{}", endpoint, key)
    }
    
    fn storage(&self) -> web_sys::Storage {
        let window = web_sys::window().unwrap();
        match self.location {
            WebStorageLocation::LocalStorage => window.local_storage().unwrap().unwrap(),
            WebStorageLocation::SessionStorage => window.session_storage().unwrap().unwrap(),
        }
    }
}

impl Vault for VaultWeb {
    fn new(location: &str) -> Self {
        let location = location.parse().expect("invalid vault_location for web, must be one of 'local_storage' or 'session_storage'");
        VaultWeb { location }
    }

    fn store(&self, endpoint: &str, key: &str, data: &str) {
        let storage = self.storage();

        storage.set_item(&self.storage_key(endpoint, key), data).unwrap();
    }

    fn get(&self, endpoint: &str, key: &str) -> Option<String> {
        let storage = self.storage();

        storage.get_item(&self.storage_key(endpoint, key)).unwrap()
    }

    fn delete(&self, endpoint: &str, key: &str) {
        let storage = self.storage();

        storage.remove_item(&self.storage_key(endpoint, key)).unwrap();
    }
}