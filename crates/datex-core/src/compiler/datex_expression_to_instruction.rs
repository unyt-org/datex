use crate::{
    ast::expressions::DatexExpression,
    compiler::context::CompilationContext,
    core_compiler::to_instructions::{
        InstructionContext, SharedValueTrackingProvider, ToInstructions,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
};
use core::cell::RefCell;
impl<'ctx, T> ToInstructions<'ctx, T> for DatexExpression
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType = RegularInstruction;
    fn to_instructions(
        &self,
        ctx: &T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>> {
        Box::new(self.data().to_instructions(ctx))
    }
}
