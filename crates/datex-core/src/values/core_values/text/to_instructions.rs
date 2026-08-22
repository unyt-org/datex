use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{InstructionContext, ToInstructions},
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::text::Text,
};

impl ToInstructions for Text {
    type InstructionType = RegularInstruction;
    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(core::iter::once(RegularInstruction::text(self.0.clone())))
    }
}
