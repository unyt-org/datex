use binrw::{BinRead, BinWrite};
use core::{fmt::Display, prelude::rust_2024::*};
use num_enum::TryFromPrimitive;

#[derive(
    Clone, Debug, PartialEq, Copy, BinWrite, BinRead, TryFromPrimitive,
)]
#[brw(little, repr(u8))]
#[repr(u8)]
pub enum AssignmentOperator {
    AddAssign,        // +=
    SubtractAssign,   // -=
    MultiplyAssign,   // *=
    DivideAssign,     // /=
    ModuloAssign,     // %=
    PowerAssign,      // ^=
    BitwiseAndAssign, // &=
    BitwiseOrAssign,  // |=
}
impl Display for AssignmentOperator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(
            f,
            "{}",
            match self {
                AssignmentOperator::AddAssign => "+=",
                AssignmentOperator::SubtractAssign => "-=",
                AssignmentOperator::MultiplyAssign => "*=",
                AssignmentOperator::DivideAssign => "/=",
                AssignmentOperator::ModuloAssign => "%=",
                AssignmentOperator::PowerAssign => "^=",
                AssignmentOperator::BitwiseAndAssign => "&=",
                AssignmentOperator::BitwiseOrAssign => "|=",
            }
        )
    }
}
