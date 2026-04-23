use crate::{
    serde::Deserialize,
    shared_values::shared_containers::SharedContainerMutability,
};
use binrw::{BinRead, BinWrite};
use core::{cmp::Ordering, fmt::Display};
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

impl Display for SharedContainerOwnership {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SharedContainerOwnership::Owned => write!(f, ""),
            SharedContainerOwnership::Referenced(mutability) => {
                match mutability {
                    ReferenceMutability::Immutable => write!(f, "'"),
                    ReferenceMutability::Mutable => write!(f, "'mut"),
                }
            }
        }
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
