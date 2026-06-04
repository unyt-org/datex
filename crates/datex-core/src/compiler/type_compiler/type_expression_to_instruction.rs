use crate::{
    ast::type_expressions::{TypeExpression, TypeExpressionData},
    core_compiler::to_instructions::ToInstructions,
    global::protocol_structures::type_instructions::TypeInstruction,
    types::literal_type_definition::LiteralTypeDefinition,
};

impl<'a> ToInstructions<'a> for TypeExpression {
    type InstructionType = TypeInstruction;
    fn to_instructions<'a>(
        &self,
    ) -> Box<dyn Iterator<Item = TypeInstruction> + '_> {
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
                    let end_instructions =
                        range.end.to_instructions(shared_value_tracking);
                    let start_instructions =
                        range.start.to_instructions(shared_value_tracking);
                    yield TypeInstruction::TypeDefinitionRange;
                    for instr in end_instructions {
                        yield instr;
                    }
                    for instr in start_instructions {
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
