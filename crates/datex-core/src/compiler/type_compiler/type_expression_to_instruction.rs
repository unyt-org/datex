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
impl ToInstructions for TypeExpression {
    type InstructionType = TypeInstruction;
    fn to_instructions<'a>(
        &'a self,
        _shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
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
