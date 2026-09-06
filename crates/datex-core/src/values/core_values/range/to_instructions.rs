use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::range::Range,
};

impl ToInstructions for Range {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(gen move {
            yield RegularInstruction::range().into();
            for instr in self.start.to_instructions(ctx) {
                yield instr;
            }
            for instr in self.end.to_instructions(ctx) {
                yield instr;
            }
        })
    }
}
