use crate::{
    instruction::{
        Instruction, regular_instruction::RegularInstruction,
        type_instruction::TypeInstruction,
    },
    values::{value::Value, value_container::ValueContainer},
};

pub enum InstructionInput {
    Instruction(Instruction),
    ValueContainer(ValueContainer),
}

impl From<Instruction> for InstructionInput {
    fn from(instruction: Instruction) -> Self {
        InstructionInput::Instruction(instruction)
    }
}
impl From<RegularInstruction> for InstructionInput {
    fn from(instruction: RegularInstruction) -> Self {
        InstructionInput::Instruction(Instruction::Regular(instruction))
    }
}
impl From<TypeInstruction> for InstructionInput {
    fn from(instruction: TypeInstruction) -> Self {
        InstructionInput::Instruction(Instruction::Type(instruction))
    }
}
impl From<ValueContainer> for InstructionInput {
    fn from(value_container: ValueContainer) -> Self {
        InstructionInput::ValueContainer(value_container)
    }
}
impl From<Value> for InstructionInput {
    fn from(value: Value) -> Self {
        InstructionInput::ValueContainer(ValueContainer::Local(value))
    }
}
