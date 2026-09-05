use crate::{
    core_compiler::into_regular_instruction::IntoRegularInstruction,
    instruction::regular_instruction::RegularInstruction,
};

impl IntoRegularInstruction for String {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::text(self.clone())
    }
}
