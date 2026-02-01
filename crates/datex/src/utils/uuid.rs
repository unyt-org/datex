use crate::{crypto::uuid::generate_uuid_string, compat::string::String};
use core::{fmt::Display, prelude::rust_2024::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UUID(String);

impl UUID {
    pub(crate) fn new() -> UUID {
        UUID(generate_uuid_string())
    }
    pub fn from_string(uuid: String) -> UUID {
        UUID(uuid)
    }
}

impl Display for UUID {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "{}", self.0)
    }
}
