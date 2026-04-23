use crate::{
    serde::Deserialize,
    shared_values::shared_containers::SharedContainerMutability,
};
use binrw::{BinRead, BinWrite};
use core::{
    cmp::Ordering,
    fmt,
    fmt::{Display, Formatter},
};
use num_enum::TryFromPrimitive;
use serde::Serialize;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TryFromPrimitive,
    Serialize,
    Deserialize,
    BinRead,
    BinWrite,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SharedContainerOwnership {
    Owned,
    Referenced(ReferenceMutability),
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
