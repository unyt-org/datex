use crate::{
    core_compiler::to_instructions::{
        SharedValueTrackingProvider, ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::endpoint::Endpoint,
};

impl<'ctx, T> ToInstructions<'ctx, T> for Endpoint
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType = RegularInstruction;
    fn to_instructions(
        &self,
        _ctx: &mut T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>> {
        Box::new(core::iter::once(RegularInstruction::endpoint(self.clone())))
    }
}
