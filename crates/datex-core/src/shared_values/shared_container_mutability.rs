use binrw::{BinRead, BinWrite};
use core::fmt::Display;
use num_enum::TryFromPrimitive;

use serde_repr::*;
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    TryFromPrimitive,
    BinRead,
    Copy,
    BinWrite,
    Serialize_repr,
    Deserialize_repr,
)]
#[brw(repr(u8))]
#[repr(u8)]
pub enum SharedContainerMutability {
    Immutable = 0,
    Mutable = 1,
}

impl Display for SharedContainerMutability {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SharedContainerMutability::Immutable => write!(f, ""),
            SharedContainerMutability::Mutable => write!(f, "mut"),
        }
    }
}
