use crate::{
    ast::type_expressions::{TypeExpression, TypeExpressionData},
    compiler::context::CompilationContext,
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::{
            InstructionContext, SharedValueTrackingProvider, ToInstructions,
        },
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    types::literal_type_definition::LiteralTypeDefinition,
};
use core::cell::RefCell;
impl<'ctx, T> ToInstructions<'ctx, T> for TypeExpression
where
    T: SharedValueTrackingProvider<'ctx>,
{
    type InstructionType = TypeInstruction;
    fn to_instructions(
        &self,
        ctx: &T,
    ) -> Box<impl Iterator<Item = Self::InstructionType>> {
        Box::new(gen move {
            match self.data() {
                TypeExpressionData::Integer(integer) => {
                    yield TypeInstruction::Literal(
                        LiteralTypeDefinition::Integer(integer.clone()),
                    )
                }
                TypeExpressionData::Text(text) => {
                    yield TypeInstruction::Literal(LiteralTypeDefinition::Text(
                        text.clone(),
                    ))
                }
                TypeExpressionData::Boolean(boolean) => {
                    yield TypeInstruction::Literal(
                        LiteralTypeDefinition::Boolean(boolean.clone()),
                    )
                }
                TypeExpressionData::GetCoreLibType(core_lib_id) => {
                    yield TypeInstruction::CoreType(*core_lib_id)
                }
                TypeExpressionData::Range(range) => {
                    yield TypeInstruction::Range;
                    for instr in
                        range.start.to_instructions(ctx).collect::<Vec<_>>()
                    {
                        yield instr;
                    }
                    for instr in
                        range.end.to_instructions(ctx).collect::<Vec<_>>()
                    {
                        yield instr;
                    }
                }
                e => todo!(
                    "Type expression to instruction not implemented for {:?}",
                    e
                ),
            }
        })
    }
}
