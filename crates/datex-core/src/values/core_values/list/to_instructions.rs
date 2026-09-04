use crate::{
    core_compiler::to_instructions::{
        ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::list::List,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for List
where
    T: ValueVisitor<'ctx>,
{
    fn to_instructions(
        &self,
        _ctx: &mut T,
    ) -> Box<impl Iterator<Item = Instruction>> {
        Box::new(gen move {
            yield RegularInstruction::list(self.items.len() as u32).into();
            for _item in &self.items {
                todo!("Implement instruction generation for value container");
                // for instruction in item.to_instructions(ctx) {
                //     yield instruction;
                // }
            }
        })
    }
}
