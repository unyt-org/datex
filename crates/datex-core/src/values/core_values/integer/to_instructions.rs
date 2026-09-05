use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    values::core_values::integer::Integer,
};

impl<'ctx, T> ToInstructions<'ctx, T> for Integer
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
        core::iter::once(Instruction::Regular(RegularInstruction::integer(
            self.clone(),
        )))
    }
}
