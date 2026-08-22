//! This module contains the definition of the [StackIndex] structure.
//! The index is used to represent the position of a value in the stack during execution.
use binrw::{BinRead, BinWrite};
use core::{fmt::Display, ops::AddAssign};

#[derive(
    BinRead, BinWrite, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[brw(little)]
pub struct StackIndex(pub u32);

impl Display for StackIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{{{}}}", self.0)
    }
}

impl AddAssign<u32> for StackIndex {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}
