use crate::utils::uuid::UUID;
use core::fmt::{Debug, Display};

use crate::prelude::*;
pub mod error;
pub mod factory;
pub mod properties;
pub mod socket;

#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm_runtime", tsify(type = "string"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComInterfaceUUID(UUID);

#[allow(clippy::new_without_default)]
impl ComInterfaceUUID {
    pub fn new() -> Self {
        ComInterfaceUUID(UUID::new())
    }
    pub fn uuid_string(&self) -> String {
        self.0.to_string()
    }
}

impl Display for ComInterfaceUUID {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "com_interface::{}", self.0)
    }
}

impl TryFrom<String> for ComInterfaceUUID {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.strip_prefix("com_interface::").ok_or(())?;
        Ok(ComInterfaceUUID(UUID::from_string(value.to_string())))
    }
}
