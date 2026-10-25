use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    types::entities::entity_type_definition::EntityTypeDefinition,
};
use crate::instruction::regular_instruction::RegularInstruction;

impl ToInstructions for EntityTypeDefinition {
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::entity_type_definition(
                self.name.to_string(),
            ).into();

            for item in self.definition.to_bytecode_items(visitor) {
                yield item;
            }
        })
    }
}
