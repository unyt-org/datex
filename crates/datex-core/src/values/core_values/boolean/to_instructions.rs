use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    values::core_values::boolean::Boolean,
};

impl<'ctx, T> ToInstructions<'ctx, T> for Boolean
where
    T: ValueVisitor<'ctx> + ?Sized,
{
    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a
    where
        'ctx: 'a,
    {
        self.0.to_instructions(ctx)
    }
}
