//! # Binary Operators for the Language
//! 
//! This module defines all binary operators available in the language,
//! along with their conversion to instruction codes for the VM.
//! 
//! ## Operator Categories
//! 
//! | Category | Operators |
//! |----------|-----------|
//! | Arithmetic | `+`, `-`, `*`, `/`, `%`, `^` |
//! | Logical | `and`, `or` |
//! | Bitwise | `&`, `\|`, `~` |
//! | Range | `..`, `..=` |
//! 
//! ## Instruction Code Mapping
//! 
//! Each operator maps to an [`InstructionCode`] for the VM:
//! 
//! | Operator | Instruction |
//! |----------|-------------|
//! | `+` | `ADD` |
//! | `-` | `SUBTRACT` |
//! | `*` | `MULTIPLY` |
//! | `/` | `DIVIDE` |
//! | `%` | `MODULO` |
//! | `^` (power) | `POWER` |
//! | `and` | `AND` |
//! | `or` | `OR` |
//! | `&` | `AND` (bitwise) |
//! | `\|` | `OR` (bitwise) |
//! | `~` | `NOT` |
//! | `^` (XOR) | Not implemented |
//! | `..` | Not implemented yet |
//! | `..=` | `Range` |
//! 
//! The Bitwise form is doing exactly the same, its just an other variant or writing the same.

use core::fmt::Display;

use crate::global::{
    instruction_codes::InstructionCode,
    protocol_structures::instructions::RegularInstruction,
};

use crate::prelude::*;
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum BinaryOperator {
    Arithmetic(ArithmeticOperator),
    Logical(LogicalOperator),
    Bitwise(BitwiseOperator),
    Range(RangeOperator),
}
impl From<ArithmeticOperator> for BinaryOperator {
    fn from(op: ArithmeticOperator) -> Self {
        BinaryOperator::Arithmetic(op)
    }
}
impl From<LogicalOperator> for BinaryOperator {
    fn from(op: LogicalOperator) -> Self {
        BinaryOperator::Logical(op)
    }
}
impl From<BitwiseOperator> for BinaryOperator {
    fn from(op: BitwiseOperator) -> Self {
        BinaryOperator::Bitwise(op)
    }
}

#[derive(Clone, Debug, PartialEq, Copy, Eq, Hash)]
pub enum ArithmeticOperator {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
    Power,    // ^
}

impl Display for ArithmeticOperator {
    /// Allow to print Arithmetic operator as string
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(
            f,
            "{}",
            match self {
                ArithmeticOperator::Add => "+",
                ArithmeticOperator::Subtract => "-",
                ArithmeticOperator::Multiply => "*",
                ArithmeticOperator::Divide => "/",
                ArithmeticOperator::Modulo => "%",
                ArithmeticOperator::Power => "^",
            }
        )
    }
}

impl From<&ArithmeticOperator> for InstructionCode {
    /// Converts a Arithmetic operator to its corresponding instruction code.
    fn from(op: &ArithmeticOperator) -> Self {
        match op {
            ArithmeticOperator::Add => InstructionCode::ADD,
            ArithmeticOperator::Subtract => InstructionCode::SUBTRACT,
            ArithmeticOperator::Multiply => InstructionCode::MULTIPLY,
            ArithmeticOperator::Divide => InstructionCode::DIVIDE,
            ArithmeticOperator::Modulo => InstructionCode::MODULO,
            ArithmeticOperator::Power => InstructionCode::POWER,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum LogicalOperator {
    And, // and
    Or,  // or
}

impl From<&LogicalOperator> for InstructionCode {
    /// Converts a Logical operator to its corresponding instruction code.
    fn from(op: &LogicalOperator) -> Self {
        match op {
            LogicalOperator::And => InstructionCode::AND,
            LogicalOperator::Or => InstructionCode::OR,
        }
    }
}

impl Display for LogicalOperator {
    /// Allows printing `LogicalOperator` values as strings.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(
            f,
            "{}",
            match self {
                LogicalOperator::And => "and",
                LogicalOperator::Or => "or",
            }
        )
    }
}

/// Bitwise operators for binary-level operations.
/// 
/// # Operators
/// 
/// | Operator | Name | Example |
/// |----------|------|---------|
/// | `&` | Bitwise AND | `a & b` |
/// | `\|` | Bitwise OR | `a \| b` |
/// | `^` | Bitwise XOR | `a ^ b` |
/// | `~` | Bitwise NOT | `~a` |
/// 
/// # Note
/// XOR (`^`) is currently unimplemented and will panic at runtime.
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum BitwiseOperator {
    And, // &
    Or,  // |
    Xor, // ^
    Not, // ~
}

/// Range operators for creating sequences.
/// This is very similar to Rust code
/// 
/// # Operators
/// 
/// | Operator | Name | Example |
/// |----------|------|---------|
/// | `..` | Exclusive range | `1..10` (1 to 9) |
/// | `..=` | Inclusive range | `1..=10` (1 to 10) |
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum RangeOperator {
    /// Exclusive range (start..end) - end is NOT included
    Exclusive, // ..
    
    /// Inclusive range (start..=end) - end is included
    Inclusive, // ..=
}

impl From<&RangeOperator> for InstructionCode {
    /// Converts a RangeOperator operator to its corresponding instruction code.
    fn from(op: &RangeOperator) -> Self {
        match op {
            RangeOperator::Inclusive => InstructionCode::RANGE,
            _ => {
                core::todo!(
                    "Bitwise operator {:?} not implemented for InstructionCode",
                    op
                )
            }
        }
    }
}

impl Display for RangeOperator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(
            f,
            "{}",
            match self {
                RangeOperator::Exclusive => "..",
                RangeOperator::Inclusive => "..=",
            }
        )
    }
}

impl From<&BitwiseOperator> for InstructionCode {
    /// Converts a bitwise operator to its corresponding instruction code.
    fn from(op: &BitwiseOperator) -> Self {
        match op {
            BitwiseOperator::And => InstructionCode::AND,
            BitwiseOperator::Or => InstructionCode::OR,
            BitwiseOperator::Not => InstructionCode::NOT,
            _ => {
                core::todo!(
                    "Bitwise operator {:?} not implemented for InstructionCode",
                    op
                )
            }
        }
    }
}

impl Display for BitwiseOperator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(
            f,
            "{}",
            match self {
                BitwiseOperator::And => "&",
                BitwiseOperator::Or => "|",
                BitwiseOperator::Xor => "^",
                BitwiseOperator::Not => "~",
            }
        )
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(
            f,
            "{}",
            match self {
                BinaryOperator::Arithmetic(op) => op.to_string(),
                BinaryOperator::Logical(op) => op.to_string(),
                BinaryOperator::Bitwise(op) => op.to_string(),
                BinaryOperator::Range(op) => op.to_string(),
            }
        )
    }
}

impl From<&BinaryOperator> for InstructionCode {
    fn from(op: &BinaryOperator) -> Self {
        match op {
            BinaryOperator::Arithmetic(op) => InstructionCode::from(op),
            BinaryOperator::Logical(op) => InstructionCode::from(op),
            BinaryOperator::Bitwise(op) => InstructionCode::from(op),
            BinaryOperator::Range(op) => InstructionCode::from(op),
        }
    }
}

impl From<BinaryOperator> for InstructionCode {
    fn from(op: BinaryOperator) -> Self {
        InstructionCode::from(&op)
    }
}

impl From<&InstructionCode> for BinaryOperator {
    fn from(code: &InstructionCode) -> Self {
        match code {
            InstructionCode::ADD => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Add)
            }
            InstructionCode::SUBTRACT => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Subtract)
            }
            InstructionCode::MULTIPLY => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Multiply)
            }
            InstructionCode::DIVIDE => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Divide)
            }
            InstructionCode::MODULO => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Modulo)
            }
            InstructionCode::POWER => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Power)
            }
            InstructionCode::AND => {
                BinaryOperator::Logical(LogicalOperator::And)
            }
            InstructionCode::OR => BinaryOperator::Logical(LogicalOperator::Or),
            InstructionCode::UNION => {
                BinaryOperator::Bitwise(BitwiseOperator::And)
            }
            InstructionCode::RANGE => {
                BinaryOperator::Range(RangeOperator::Inclusive)
            }
            _ => core::todo!(
                "#154 Binary operator for {:?} not implemented",
                code
            ),
        }
    }
}

impl From<InstructionCode> for BinaryOperator {
    fn from(code: InstructionCode) -> Self {
        BinaryOperator::from(&code)
    }
}

impl From<&RegularInstruction> for BinaryOperator {
    fn from(instruction: &RegularInstruction) -> Self {
        match instruction {
            RegularInstruction::Add => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Add)
            }
            RegularInstruction::Subtract => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Subtract)
            }
            RegularInstruction::Multiply => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Multiply)
            }
            RegularInstruction::Divide => {
                BinaryOperator::Arithmetic(ArithmeticOperator::Divide)
            }
            RegularInstruction::Range => {
                BinaryOperator::Range(RangeOperator::Inclusive)
            }
            RegularInstruction::And => {
                BinaryOperator::Logical(LogicalOperator::And)
            }
            RegularInstruction::Or => {
                BinaryOperator::Logical(LogicalOperator::Or)
            }
            _ => {
                core::todo!(
                    "#155 Binary operator for instruction {:?} not implemented",
                    instruction
                );
            }
        }
    }
}

impl From<RegularInstruction> for BinaryOperator {
    fn from(instruction: RegularInstruction) -> Self {
        BinaryOperator::from(&instruction)
    }
}
