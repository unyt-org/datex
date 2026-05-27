use core::time::Duration;

#[derive(Debug)]
pub struct ComHubOptions {
    pub default_receive_timeout: Duration,
    pub keys: Option<String>
}
impl ComHubOptions {
    pub fn not_default(keys: Option<String>) -> Self {
        ComHubOptions {
            default_receive_timeout: Duration::from_secs(5),
            keys,
        }
    }
}

impl Default for ComHubOptions {
    fn default() -> Self {
        ComHubOptions {
            default_receive_timeout: Duration::from_secs(5),
            keys: None,
        }
    }
}
