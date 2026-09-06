use crate::{
    core_compiler::into_regular_instruction::{
        IntoRegularInstruction, impl_regular_to_instructions,
    },
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

impl_regular_to_instructions!(f32, f64);
