use alloc::vec::Vec;

/// This trait represents a persistent data storage which can be used to store raw bytes with an associated key for a specific endpoint.
/// It is used to store small data that needs to be persisted across multiple executions, like endpoint keys.
pub trait Vault {
    /// Store raw bytes in the vault for a specific endpoint and key.
    fn store(&self, endpoint: &str, key: &str, data: &[u8]);
    /// Retrieve raw bytes from the vault for a specific endpoint and key.
    fn get(&self, endpoint: &str, key: &str) -> Option<Vec<u8>>;
    /// Delete raw bytes from the vault for a specific endpoint and key.
    fn delete(&self, endpoint: &str, key: &str);
}