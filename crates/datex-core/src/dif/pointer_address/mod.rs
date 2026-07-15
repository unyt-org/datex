pub mod serde_dif;
use core::fmt::Display;
use serde::de::Error;
use crate::shared_values::{PointerAddress, SharedContainerOwnership};

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

impl TryFrom<&str> for PointerAddressWithOwnership {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (prefix, address_str) = match value.split_once('$') {
            Some((prefix, address_str)) => (prefix, address_str),
            None => ("", value),
        };
        let address = PointerAddress::try_from(address_str).map_err(|e| {
            format!("invalid pointer address: {}", e)
        })?;
        let ownership = SharedContainerOwnership::try_from_string(prefix)
            .ok_or_else(|| {
                format!("invalid ownership: {}", value)
            })?;
        Ok(PointerAddressWithOwnership { address, ownership })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        dif::pointer_address::PointerAddressWithOwnership,
        shared_values::{
            PointerAddress, ReferenceMutability, SelfOwnedPointerAddress,
            SharedContainerOwnership,
        },
    };
    use alloc::string::ToString;
    use test_case::test_case;

    #[test_case(PointerAddressWithOwnership {
        address: PointerAddress::SelfOwned(SelfOwnedPointerAddress::new([1u8, 2u8, 3u8, 4u8, 5u8])),
        ownership: SharedContainerOwnership::Referenced(ReferenceMutability::Mutable),
    }, "'mut$0102030405" ; "mutable")]
    #[test_case(PointerAddressWithOwnership {
        address: PointerAddress::SelfOwned(SelfOwnedPointerAddress::new([1u8, 2u8, 3u8, 4u8, 5u8])),
        ownership: SharedContainerOwnership::Referenced(ReferenceMutability::Immutable),
    }, "'$0102030405" ; "immutable")]
    #[test_case(PointerAddressWithOwnership {
        address: PointerAddress::SelfOwned(SelfOwnedPointerAddress::new([1u8, 2u8, 3u8, 4u8, 5u8])),
        ownership: SharedContainerOwnership::Owned,
    }, "$0102030405" ; "owned")]
    fn display(
        pointer_with_ownership: PointerAddressWithOwnership,
        expected: &str,
    ) {
        assert_eq!(pointer_with_ownership.to_string(), expected);
    }
}
