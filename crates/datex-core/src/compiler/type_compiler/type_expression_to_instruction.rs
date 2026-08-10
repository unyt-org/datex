use crate::{
    ast::type_expressions::{TypeExpression, TypeExpressionData},
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::ToInstructions,
    },
    global::protocol_structures::type_instructions::TypeInstruction,
    prelude::*,
    types::literal_type_definition::LiteralTypeDefinition,
};
impl<'a> ToInstructions<'a> for TypeExpression {
    type InstructionType = TypeInstruction;
    fn to_instructions(
        &'a self,
        _shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            match self.data() {
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
                TypeExpressionData::GetCoreLibType(core_lib_id) => {
                    yield TypeInstruction::TypeDefinitionCoreType(
                        core_lib_id.clone(),
                    )
                }
                TypeExpressionData::Range(range) => {
                    yield TypeInstruction::TypeDefinitionRange;
                    for instr in
                        range.start.to_instructions(_shared_value_tracking)
                    {
                        yield instr;
                    }
                    for instr in
                        range.end.to_instructions(_shared_value_tracking)
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
