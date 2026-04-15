use core::cmp::Ordering;
use binrw::{BinRead, BinWrite};
use num_enum::TryFromPrimitive;
use serde::Serialize;
use crate::serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, Serialize, Deserialize, BinRead, BinWrite, PartialOrd)]
#[brw(repr(u8))]
#[repr(u8)]
pub enum ReferenceMutability {
    Immutable = 0,
    Mutable = 1,
}

impl Ord for ReferenceMutability {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ReferenceMutability::Immutable, ReferenceMutability::Immutable) => Ordering::Equal,
            (ReferenceMutability::Immutable, ReferenceMutability::Mutable) => Ordering::Less,
            (ReferenceMutability::Mutable, ReferenceMutability::Immutable) => Ordering::Greater,
            (ReferenceMutability::Mutable, ReferenceMutability::Mutable) => Ordering::Equal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub enum SharedContainerOwnership {
    Owned,
    Referenced(ReferenceMutability),
}

impl Ord for SharedContainerOwnership {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (SharedContainerOwnership::Owned, SharedContainerOwnership::Owned) => Ordering::Equal,
            (SharedContainerOwnership::Owned, SharedContainerOwnership::Referenced(_)) => Ordering::Greater,
            (SharedContainerOwnership::Referenced(_), SharedContainerOwnership::Owned) => Ordering::Less,
            (SharedContainerOwnership::Referenced(m1), SharedContainerOwnership::Referenced(m2)) => m1.cmp(m2),
        }
    }
}