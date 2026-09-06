use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, ToInstructionsDyn},
        value_visitor::ValueVisitor,
    },
    instruction::Instruction,
};

impl<V> ToInstructions for Box<V>
where
    V: ToInstructions + ?Sized,
{
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        self.as_ref().to_instructions(ctx)
    }
}
