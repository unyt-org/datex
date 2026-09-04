use crate::{
    ast::expressions::DatexExpression,
    core_compiler::to_instructions::{
        ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for DatexExpression
where
    T: ValueVisitor<'ctx>,
{

    fn to_instructions(
        &self,
        ctx: &mut T,
    ) -> Box<impl Iterator<Item = Instruction>> {
        Box::new(self.data().to_instructions(ctx))
    }
}
