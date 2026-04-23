pub mod serde;
use core::fmt::Display;

use crate::shared_values::{
    pointer_address::PointerAddress,
    shared_containers::SharedContainerOwnership,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerAddressWithOwnership {
    pub address: PointerAddress,
    pub ownership: SharedContainerOwnership,
}
impl Display for PointerAddressWithOwnership {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}", self.ownership, self.address)
    }
}
