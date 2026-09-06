use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
};

impl ToInstructions for bool {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        _ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(core::iter::once(RegularInstruction::boolean(*self).into()))
    }
}
