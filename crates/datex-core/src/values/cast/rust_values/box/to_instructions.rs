use crate::{
    core_compiler::{
        into_regular_instruction::IntoRegularInstruction,
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
};

impl<'ctx, C, V> ToInstructions<'ctx, C> for Box<V>
where
    C: ValueVisitor<'ctx> + ?Sized,
    V: ToInstructions<'ctx, C> + ?Sized,
{
    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut C,
    ) -> impl Iterator<Item = Instruction> + 'a
    where
        'ctx: 'a,
    {
        self.as_ref().to_instructions(ctx)
    }
}
