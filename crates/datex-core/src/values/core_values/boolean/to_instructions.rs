use crate::{
    core_compiler::to_instructions::{
        SharedValueTrackingProvider, ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::boolean::Boolean,
};

impl<'ctx, T> ToInstructions<'ctx, T> for Boolean
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType = RegularInstruction;
    fn to_instructions(
        &self,
        _ctx: &mut T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>> {
        Box::new(core::iter::once(if self.is_true() {
            RegularInstruction::r#true()
        } else {
            RegularInstruction::r#false()
        }))
    }
}
