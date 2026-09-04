use crate::{
    core_compiler::to_instructions::{
        ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::Instant,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for Instant
where
    T: ValueVisitor<'ctx>,
{

    fn to_instructions<'a>(
        &'a self,
        _ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        core::iter::once(RegularInstruction::instant(self.0).into())
    }
}
