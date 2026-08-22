use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{InstructionContext, ToInstructions},
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::endpoint::Endpoint,
};

use crate::prelude::*;
impl ToInstructions for Endpoint {
    type InstructionType = RegularInstruction;
    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(core::iter::once(RegularInstruction::endpoint(self.clone())))
    }
}
