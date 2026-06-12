#[derive(Debug, Clone)]
pub struct UnexpectedPropertyError {
    pub key: String,
}

impl core::fmt::Display for UnexpectedPropertyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Unexpected property: {:?}", self.key)
    }
}
