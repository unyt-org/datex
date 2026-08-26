use crate::{
    ast::expressions::DatexExpression,
    core_compiler::to_instructions::{InstructionContext, ToInstructions},
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
};
impl ToInstructions for DatexExpression {
    type InstructionType = RegularInstruction;
    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(self.data().to_instructions(ctx))
    }
}
