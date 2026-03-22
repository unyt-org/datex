use core::{fmt::Display, prelude::rust_2024::*};
use crate::global::protocol_structures::regular_instructions::RegularInstruction;
use crate::global::protocol_structures::type_instructions::TypeInstruction;

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    // regular instruction
    RegularInstruction(RegularInstruction),
    // Type instruction that yields a type
    TypeInstruction(TypeInstruction),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Instruction::RegularInstruction(instr) => {
                write!(f, "{}", instr)
            }
            Instruction::TypeInstruction(instr) => {
                write!(f, "TYPE_INSTRUCTION {}", instr)
            }
        }
    }
}

impl From<RegularInstruction> for Instruction {
    fn from(instruction: RegularInstruction) -> Self {
        Instruction::RegularInstruction(instruction)
    }
}

impl From<TypeInstruction> for Instruction {
    fn from(instruction: TypeInstruction) -> Self {
        Instruction::TypeInstruction(instruction)
    }
}