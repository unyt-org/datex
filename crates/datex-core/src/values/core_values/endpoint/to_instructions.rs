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
    T: ValueVisitor<'ctx> + ?Sized,
{
    fn to_instructions<'a>(
        &'a self,
        _ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(core::iter::once(RegularInstruction::endpoint(self.clone()).into()))
    }
}
