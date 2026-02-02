use crate::prelude::*;
use core::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UUID(String);

impl UUID {
    pub(crate) fn new() -> UUID {
        UUID("FIXME".to_string())
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
