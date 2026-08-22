use crate::shared_values::SharedContainerMutability;
use binrw::{BinRead, BinWrite};
use core::{
    cmp::Ordering,
    fmt,
    fmt::{Display, Formatter},
};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

use serde_repr::*;
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TryFromPrimitive,
    BinRead,
    BinWrite,
    Serialize_repr,
    Deserialize_repr,
)]
#[brw(repr(u8))]
#[repr(u8)]
pub enum ReferenceMutability {
    Immutable = 0,
    Mutable = 1,
}
impl ReferenceMutability {
    pub const fn string(&self) -> &'static str {
        match self {
            ReferenceMutability::Immutable => "'",
            ReferenceMutability::Mutable => "'mut",
        }
    }
}

impl Display for ReferenceMutability {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string())
    }
}

impl From<ReferenceMutability> for SharedContainerOwnership {
    fn from(mutability: ReferenceMutability) -> Self {
        SharedContainerOwnership::Referenced(mutability)
    }
}

impl From<ReferenceMutability> for SharedContainerMutability {
    fn from(mutability: ReferenceMutability) -> Self {
        match mutability {
            ReferenceMutability::Immutable => {
                SharedContainerMutability::Immutable
            }
            ReferenceMutability::Mutable => SharedContainerMutability::Mutable,
        }
    }
}

impl PartialOrd<Self> for ReferenceMutability {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ReferenceMutability {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                ReferenceMutability::Immutable,
                ReferenceMutability::Immutable,
            ) => Ordering::Equal,
            (ReferenceMutability::Immutable, ReferenceMutability::Mutable) => {
                Ordering::Less
            }
            (ReferenceMutability::Mutable, ReferenceMutability::Immutable) => {
                Ordering::Greater
            }
            (ReferenceMutability::Mutable, ReferenceMutability::Mutable) => {
                Ordering::Equal
            }
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    BinRead,
    BinWrite,
)]
#[serde(untagged)]
pub enum SharedContainerOwnership {
    #[brw(magic = 0x0u8)]
    Owned,
    #[brw(magic = 0x1u8)]
    Referenced(ReferenceMutability),
}

impl TryFrom<Option<u8>> for SharedContainerOwnership {
    type Error = &'static str;
    fn try_from(value: Option<u8>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(SharedContainerOwnership::Owned),
            Some(i) => {
                let mutability = ReferenceMutability::try_from(i)
                    .map_err(|_| "Invalid ownership value")?;
                Ok(SharedContainerOwnership::Referenced(mutability))
            }
        }
    }
}

impl SharedContainerOwnership {
    const OWNED: &'static str = SharedContainerOwnership::Owned.string();
    const REFERENCED_IMMUTABLE: &'static str =
        SharedContainerOwnership::Referenced(ReferenceMutability::Immutable)
            .string();
    const REFERENCED_MUTABLE: &'static str =
        SharedContainerOwnership::Referenced(ReferenceMutability::Mutable)
            .string();

    pub const fn string(&self) -> &'static str {
        match self {
            SharedContainerOwnership::Owned => "",
            SharedContainerOwnership::Referenced(mutability) => {
                mutability.string()
            }
        }
    }
    pub const fn try_from_string(s: &str) -> Option<Self> {
        match s {
            Self::OWNED => Some(SharedContainerOwnership::Owned),
            Self::REFERENCED_IMMUTABLE => {
                Some(SharedContainerOwnership::Referenced(
                    ReferenceMutability::Immutable,
                ))
            }
            Self::REFERENCED_MUTABLE => {
                Some(SharedContainerOwnership::Referenced(
                    ReferenceMutability::Mutable,
                ))
            }
            _ => None,
        }
    }
}

impl Display for SharedContainerOwnership {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string())
    }
}

impl PartialOrd<Self> for SharedContainerOwnership {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SharedContainerOwnership {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                SharedContainerOwnership::Owned,
                SharedContainerOwnership::Owned,
            ) => Ordering::Equal,
            (
                SharedContainerOwnership::Owned,
                SharedContainerOwnership::Referenced(_),
            ) => Ordering::Greater,
            (
                SharedContainerOwnership::Referenced(_),
                SharedContainerOwnership::Owned,
            ) => Ordering::Less,
            (
                SharedContainerOwnership::Referenced(m1),
                SharedContainerOwnership::Referenced(m2),
            ) => m1.cmp(m2),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::shared_values::{ReferenceMutability, SharedContainerOwnership};
    use test_case::test_case;
    #[test_case(SharedContainerOwnership::Owned; "owned")]
    #[test_case(SharedContainerOwnership::Referenced(ReferenceMutability::Immutable); "referenced immutable")]
    #[test_case(SharedContainerOwnership::Referenced(ReferenceMutability::Mutable); "referenced mutable")]
    fn ownership_serde(ownership: SharedContainerOwnership) {
        let serialized = serde_json::to_value(ownership).unwrap();
        let deserialized: SharedContainerOwnership =
            serde_json::from_value(serialized).unwrap();
        assert_eq!(ownership, deserialized);
    }
}
