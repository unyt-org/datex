//! The [ByteCode] item is the parent of the Instruction and hold special handle for labels and jumps as well as instructions.

use crate::{
    compiler::metadata::Label,
    instruction::{
        Instruction, regular_instruction::RegularInstruction,
        type_instruction::TypeInstruction,
    },
};
#[derive(Debug, Clone, PartialEq)]
pub enum BytecodeItem {
    Instruction(Instruction),
    Label(Label),
    Jump { target: Label },
    JumpIfFalse { target: Label, condition_end: Label },
}
impl From<Instruction> for BytecodeItem {
    fn from(instruction: Instruction) -> Self {
        Self::Instruction(instruction)
    }
}
impl From<RegularInstruction> for BytecodeItem {
    fn from(instruction: RegularInstruction) -> Self {
        Self::Instruction(Instruction::Regular(instruction))
    }
}

impl From<TypeInstruction> for BytecodeItem {
    fn from(instruction: TypeInstruction) -> Self {
        Self::Instruction(Instruction::Type(instruction))
    }
}
