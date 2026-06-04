use crate::{
    ast::type_expressions::{TypeExpression, TypeExpressionData},
    core_compiler::to_instructions::ToInstructions,
    global::protocol_structures::type_instructions::TypeInstruction,
    types::literal_type_definition::LiteralTypeDefinition,
};
use crate::core_compiler::shared_value_tracking::SharedValueTracking;

impl<'a> ToInstructions<'a> for TypeExpression {
    type InstructionType = TypeInstruction;
    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            match &self.data {
                TypeExpressionData::Integer(integer) => {
                    yield TypeInstruction::TypeDefinitionLiteral(
                        LiteralTypeDefinition::Integer(integer.clone()),
                    )
                }
                TypeExpressionData::Text(text) => {
                    yield TypeInstruction::TypeDefinitionLiteral(
                        LiteralTypeDefinition::Text(text.clone()),
                    )
                }
                TypeExpressionData::Boolean(boolean) => {
                    yield TypeInstruction::TypeDefinitionLiteral(
                        LiteralTypeDefinition::Boolean(boolean.clone()),
                    )
                }
                TypeExpressionData::Range(range) => {
                    yield TypeInstruction::TypeDefinitionRange;
                    for instr in range.start.to_instructions(shared_value_tracking) {
                        yield instr;
                    }
                    for instr in range.end.to_instructions(shared_value_tracking) {
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
