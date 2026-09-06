use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::list::List,
};
impl ToInstructions for List {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(gen move {
            yield RegularInstruction::list(self.items.len() as u32).into();
            for item in &self.items {
                for instruction in item.to_instructions(ctx) {
                    yield instruction;
                }
            }
        })
    }
}
