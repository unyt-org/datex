use core::{fmt::Display, prelude::rust_2024::*};
use crate::global::protocol_structures::regular_instructions::RegularInstruction;
use crate::global::protocol_structures::type_instructions::TypeInstruction;

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    // regular instruction
    Regular(RegularInstruction),
    // Type instruction that yields a type
    Type(TypeInstruction),
}

impl Instruction {
    pub fn get_next_expected_instructions(&self) -> NextExpectedInstructions {
        match self {
            Instruction::Regular(instr) => instr.get_next_expected_instructions(),
            Instruction::Type(instr) => instr.get_next_expected_instructions(),
        }
    }
    
    pub fn to_formatted_string(&self) -> String {
        match self {
            Instruction::Regular(instr) => instr.to_formatted_string(),
            Instruction::Type(instr) => instr.to_formatted_string(),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Instruction::Regular(instr) => {
                write!(f, "{}", instr)
            }
            Instruction::Type(instr) => {
                write!(f, "TYPE_INSTRUCTION {}", instr)
            }
        }
    }
}

impl From<RegularInstruction> for Instruction {
    fn from(instruction: RegularInstruction) -> Self {
        Instruction::Regular(instruction)
    }
}

impl From<TypeInstruction> for Instruction {
    fn from(instruction: TypeInstruction) -> Self {
        Instruction::Type(instruction)
    }
}

pub enum NextExpectedInstructions {
    None,
    Regular(u32),
    Type(u32),
    UnboundedStart,
    UnboundedEnd,
    RegularAndType(u32, u32),
}

pub enum CountOrUnbounded {
    Count(u32),
    UnboundedStart,
    UnboundedEnd,
}

impl NextExpectedInstructions {
    pub fn total_count(&self) -> Option<CountOrUnbounded> {
        match self {
            NextExpectedInstructions::None => None,
            NextExpectedInstructions::Regular(count) => Some(CountOrUnbounded::Count(*count)),
            NextExpectedInstructions::Type(count) => Some(CountOrUnbounded::Count(*count)),
            NextExpectedInstructions::UnboundedStart => Some(CountOrUnbounded::UnboundedStart),
            NextExpectedInstructions::UnboundedEnd => Some(CountOrUnbounded::UnboundedEnd),
            NextExpectedInstructions::RegularAndType(regular_count, type_count) => Some(CountOrUnbounded::Count(regular_count + type_count)),
        }
    }
}