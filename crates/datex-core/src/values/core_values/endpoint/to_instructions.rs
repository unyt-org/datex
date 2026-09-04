use crate::{
    core_compiler::to_instructions::{
        ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::endpoint::Endpoint,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for Endpoint
where
    T: ValueVisitor<'ctx>,
{

    fn to_instructions(
        &self,
        _ctx: &mut T,
    ) -> Box<impl Iterator<Item = Instruction>> {
        Box::new(core::iter::once(RegularInstruction::endpoint(self.clone()).into()))
    }
}
