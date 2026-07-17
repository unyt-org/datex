use crate::runtime::RuntimeInternal;

impl RuntimeInternal {
    /// Logs an info message with the runtime's endpoint as a prefix.
    pub fn log_info<S: AsRef<str>>(&self, message: S) {
        log::info!("[{}] {}", self.endpoint(), message.as_ref());
    }

    /// Logs an error message with the runtime's endpoint as a prefix.
    pub fn log_error<S: AsRef<str>>(&self, message: S) {
        log::error!("[{}] {}", self.endpoint(), message.as_ref());
    }

    /// Logs a debug message with the runtime's endpoint as a prefix.
    pub fn log_debug<S: AsRef<str>>(&self, message: S) {
        log::debug!("[{}] {}", self.endpoint(), message.as_ref());
    }

    /// Logs a warning message with the runtime's endpoint as a prefix.
    pub fn log_warn<S: AsRef<str>>(&self, message: S) {
        log::warn!("[{}] {}", self.endpoint(), message.as_ref());
    }
}
