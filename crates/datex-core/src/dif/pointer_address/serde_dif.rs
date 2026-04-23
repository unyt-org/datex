use crate::{
    alloc::string::ToString,
    dif::pointer_address::PointerAddressWithOwnership,
    shared_values::{PointerAddress, SharedContainerOwnership},
};

use alloc::{format, string::String};
use serde::{Deserialize, Serialize, de::Error};
impl Serialize for PointerAddressWithOwnership {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PointerAddressWithOwnership {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (prefix, address_str) = match s.split_once('$') {
            Some((prefix, address_str)) => (prefix, address_str),
            None => ("", s.as_str()),
        };
        let address = PointerAddress::try_from(address_str).map_err(|_| {
            Error::custom(format!("invalid pointer address: {}", s))
        })?;
        let ownership = SharedContainerOwnership::try_from_string(prefix)
            .ok_or_else(|| {
                Error::custom(format!("invalid ownership: {}", s))
            })?;
        Ok(PointerAddressWithOwnership { address, ownership })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        alloc::format,
        dif::pointer_address::PointerAddressWithOwnership,
        runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
        shared_values::{
            PointerAddress, ReferenceMutability, SharedContainerOwnership,
        },
    };
    use test_case::test_case;

    #[test_case(SharedContainerOwnership::Referenced(ReferenceMutability::Mutable), "'mut" ; "mutable")]
    #[test_case(SharedContainerOwnership::Referenced(ReferenceMutability::Immutable), "'" ; "immutable")]
    #[test_case(SharedContainerOwnership::Owned, "" ; "owned")]
    fn serde(ownership: SharedContainerOwnership, expected_prefix: &str) {
        let mut pointer_address_provider =
            SelfOwnedPointerAddressProvider::default();
        let pointer_address =
            pointer_address_provider.get_new_self_owned_address();
        let address = PointerAddress::SelfOwned(pointer_address);
        let pointer_with_ownership = PointerAddressWithOwnership {
            address: address.clone(),
            ownership,
        };
        let serialized =
            serde_json::to_string(&pointer_with_ownership).unwrap();
        assert_eq!(serialized, format!("\"{expected_prefix}{address}\""));
        let deserialized: PointerAddressWithOwnership =
            serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, pointer_with_ownership);
    }
}
