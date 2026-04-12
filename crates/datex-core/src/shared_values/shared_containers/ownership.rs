use binrw::{BinRead, BinWrite};
use num_enum::TryFromPrimitive;
use serde::Serialize;
use crate::serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, Serialize, Deserialize, BinRead, BinWrite)]
#[brw(repr(u8))]
#[repr(u8)]
pub enum ReferenceMutability {
    Immutable = 0,
    Mutable = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SharedContainerOwnership {
    Owned,
    Referenced(ReferenceMutability),
}