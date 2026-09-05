use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, ToInstructionsDyn},
        value_visitor::ValueVisitor,
    },
    instruction::Instruction,
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

impl<T> ToInstructionsDyn for Box<T>
where
    T: ToInstructionsDyn + ?Sized,
{
    fn to_instructions_dyn<'a, 'ctx>(
        &'a self,
        ctx: &'a mut (dyn ValueVisitor<'ctx> + 'ctx),
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(self.as_ref().to_instructions_dyn(ctx))
    }
}
