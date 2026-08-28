use binrw::{BinRead, BinWrite};
use core::{fmt::Display, prelude::rust_2024::*};
use num_enum::TryFromPrimitive;

#[derive(
    Clone, Debug, PartialEq, Copy, BinWrite, BinRead, TryFromPrimitive,
)]
#[brw(little, repr(u8))]
#[repr(u8)]
pub enum ModificationOperator {
    AddAssign,        // +=
    SubtractAssign,   // -=
    MultiplyAssign,   // *=
    DivideAssign,     // /=
    ModuloAssign,     // %=
    PowerAssign,      // ^=
    BitwiseAndAssign, // &=
    BitwiseOrAssign,  // |=
}
impl Display for ModificationOperator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(
            f,
            "{}",
            match self {
                ModificationOperator::AddAssign => "+=",
                ModificationOperator::SubtractAssign => "-=",
                ModificationOperator::MultiplyAssign => "*=",
                ModificationOperator::DivideAssign => "/=",
                ModificationOperator::ModuloAssign => "%=",
                ModificationOperator::PowerAssign => "^=",
                ModificationOperator::BitwiseAndAssign => "&=",
                ModificationOperator::BitwiseOrAssign => "|=",
            }
        )
    }
}
