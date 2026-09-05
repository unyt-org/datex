use crate::{
    core_compiler::into_regular_instruction::IntoRegularInstruction,
    instruction::regular_instruction::RegularInstruction,
};

impl IntoRegularInstruction for f32 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::decimal_f32(*self)
    }
}

impl IntoRegularInstruction for f64 {
    fn into_regular_instruction(&self) -> RegularInstruction {
        RegularInstruction::decimal_f64(*self)
    }
}
