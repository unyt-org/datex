use alloc::string::String;

/// This trait represents a persistent data storage which can be used to store raw bytes with an associated key for a specific endpoint.
/// It is used to store small data that needs to be persisted across multiple executions, like endpoint keys.
pub trait Vault {
    /// Create a new instance of the vault with a location.
    /// The location string can be interpreted by the implementation as needed, e.g., as a file path or a web storage type.
    fn new(location: &str) -> Self;

    /// Store raw bytes in the vault for a specific endpoint and key.
    fn store(&self, endpoint: &str, key: &str, data: &str);
    /// Retrieve raw bytes from the vault for a specific endpoint and key.
    fn get(&self, endpoint: &str, key: &str) -> Option<String>;
    /// Delete raw bytes from the vault for a specific endpoint and key.
    fn delete(&self, endpoint: &str, key: &str);
}