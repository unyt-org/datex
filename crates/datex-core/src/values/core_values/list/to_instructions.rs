use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    values::core_values::list::List,
};

impl<'ctx, T> ToInstructions<'ctx, T> for List
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
        gen move {
            yield RegularInstruction::list(self.items.len() as u32).into();
            for item in &self.items {
                for instruction in item.to_instructions(ctx) {
                    yield instruction;
                }
            }
        }
    }
}
