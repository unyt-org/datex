use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{
            InstructionContext, SharedValueTrackingProvider, ToInstructions,
        },
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::Instant,
};

use crate::prelude::*;
impl<'ctx, T> ToInstructions<'ctx, T> for Instant
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType = RegularInstruction;
    fn to_instructions(
        &self,
        ctx: &T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>> {
        Box::new(core::iter::once(RegularInstruction::instant(self.0)))
    }
}
