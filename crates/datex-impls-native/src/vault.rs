use std::path::{PathBuf};
use datex_facades::vault::Vault;
use std::fs;

/// A native implementation of the Vault trait that stores data in the local file system.
pub struct VaultNative {
    storage_directory: PathBuf,
}

impl VaultNative {
    pub fn storage_file_path(&self, endpoint: &str, key: &str) -> PathBuf {
        let storage_directory = &self.storage_directory;
        let endpoint_sanitized = Self::sanitize_path_component(endpoint);
        let key_sanitized = Self::sanitize_path_component(key);
        storage_directory
            .join(endpoint_sanitized)
            .join(key_sanitized)
    }
    
    fn sanitize_path_component(component: &str) -> String {
        component
            .replace("/", "_")
            .replace("\\", "_")
            .replace(":", "_")
            .replace("*", "_")
            .replace("?", "_")
            .replace("\"", "_")
            .replace("'", "_")
            .replace("<", "_")
            .replace(">", "_")
            .replace("|", "_")
            .replace(" ", "_")
    }
}


impl Vault for VaultNative {
    /// Creates a new instance of `VaultNative` with a storage directory.
    fn new(location: &str) -> Self {
        VaultNative {
            storage_directory: PathBuf::from(location),
        }
    }

    fn store(&self, endpoint: &str, key: &str, data: &str) {
        let file_path = self.storage_file_path(endpoint, key);
        if let Some(parent_dir) = file_path.parent() {
            fs::create_dir_all(parent_dir).unwrap();
        }
        fs::write(file_path, data).unwrap();
    }

    fn get(&self, endpoint: &str, key: &str) -> Option<String> {
        let file_path = self.storage_file_path(endpoint, key);
        if file_path.exists() {
            fs::read_to_string(file_path).ok()
        } else {
            None
        }
    }

    fn delete(&self, endpoint: &str, key: &str) {
        let file_path = self.storage_file_path(endpoint, key);
        if file_path.exists() {
            fs::remove_file(file_path).ok();
        }
    }
}