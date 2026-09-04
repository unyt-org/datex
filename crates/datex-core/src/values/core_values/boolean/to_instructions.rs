use crate::{
    core_compiler::to_instructions::{
        ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    values::core_values::boolean::Boolean,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for Boolean
where
    T: ValueVisitor<'ctx>,
{

    fn to_instructions(
        &self,
        _ctx: &mut T,
    ) -> Box<impl Iterator<Item = Instruction>> {
        Box::new(core::iter::once(if self.is_true() {
            RegularInstruction::r#true().into()
        } else {
            RegularInstruction::r#false().into()
        }))
    }
}
