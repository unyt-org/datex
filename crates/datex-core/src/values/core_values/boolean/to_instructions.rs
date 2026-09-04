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

    fn to_instructions<'a>(
        &'a self,
        _ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(core::iter::once(if self.is_true() {
            RegularInstruction::r#true().into()
        } else {
            RegularInstruction::r#false().into()
        }))
    }
}
