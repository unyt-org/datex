use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    values::core_values::range::Range,
};

impl<'ctx, T> ToInstructions<'ctx, T> for Range
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
            yield RegularInstruction::range().into();
            for instr in self.start.to_instructions(ctx) {
                yield instr;
            }
            for instr in self.end.to_instructions(ctx) {
                yield instr;
            }
        }
    }
}
