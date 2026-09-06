use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::Instant,
};

impl ToInstructions for Instant {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        _ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(core::iter::once(RegularInstruction::instant(self.0).into()))
    }
}
