use crate::{
    core_compiler::into_regular_instruction::IntoRegularInstruction,
    instruction::regular_instruction::RegularInstruction,
};

impl IntoRegularInstruction for bool {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::boolean(*self)
    }
}
