use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
};

impl<'ctx, T> ToInstructions<'ctx, T> for bool
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
        Box::new(core::iter::once(RegularInstruction::boolean(*self).into()))
    }
}
