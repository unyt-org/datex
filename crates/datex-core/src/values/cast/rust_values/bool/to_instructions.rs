use crate::{
    core_compiler::into_regular_instruction::ToRegularInstruction,
    instruction::regular_instruction::RegularInstruction,
};

impl ToRegularInstruction for bool {
    fn to_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::boolean(*self)
    }
}
