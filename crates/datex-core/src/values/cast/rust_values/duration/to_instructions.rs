use core::time::Duration;

use crate::{
    core_compiler::{
        into_regular_instruction::IntoRegularInstruction,
        to_instructions::{ToInstructions, ToInstructionsDyn},
        value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::Instant,
};

// impl IntoRegularInstruction for Instant {
//     fn into_regular_instruction(&self) -> RegularInstruction {
//         // TODO: Implement the conversion from Instant to RegularInstruction
//         unimplemented!()
//     }
// }

impl<'ctx, T> ToInstructions<'ctx, T> for Duration
where
    T: ValueVisitor<'ctx> + ?Sized,
{
    fn to_instructions<'a>(
        &'a self,
        _ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a
    where
        'ctx: 'a,
    {
        gen move { todo!() }
    }
}
impl ToInstructionsDyn for Duration {
    fn to_instructions_dyn<'a, 'ctx>(
        &'a self,
        ctx: &'a mut (dyn ValueVisitor<'ctx> + 'ctx),
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(self.to_instructions(ctx))
    }
}
