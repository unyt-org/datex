use crate::{
    ast::type_expressions::{TypeExpression, TypeExpressionData},
    core_compiler::to_instructions::{
        ToInstructions,
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    types::literal_type_definition::LiteralTypeDefinition,
};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for TypeExpression
where
    T: ValueVisitor<'ctx> + ?Sized,
{

    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen move {
            match self.data() {
                TypeExpressionData::Integer(integer) => {
                    yield TypeInstruction::Literal(
                        LiteralTypeDefinition::Integer(integer.clone()),
                    ).into()
                }
                TypeExpressionData::Text(text) => {
                    yield TypeInstruction::Literal(LiteralTypeDefinition::Text(
                        text.clone(),
                    )).into()
                }
                TypeExpressionData::Boolean(boolean) => {
                    yield TypeInstruction::Literal(
                        LiteralTypeDefinition::Boolean(boolean.clone()),
                    ).into()
                }
                TypeExpressionData::GetCoreLibType(core_lib_id) => {
                    yield TypeInstruction::CoreType(*core_lib_id).into()
                }
                TypeExpressionData::Range(range) => {
                    yield TypeInstruction::Range.into();
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
