use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{
            InstructionContext, SharedValueTrackingProvider, ToInstructions,
        },
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::text::Text,
};

impl<'ctx, T> ToInstructions<'ctx, T> for Text
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType = RegularInstruction;
    fn to_instructions(
        &self,
        ctx: &mut T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>> {
        Box::new(core::iter::once(RegularInstruction::text(self.0.clone())))
    }
}
