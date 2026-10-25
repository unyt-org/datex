use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    prelude::*,
    types::entities::entity_type_definition::EntityTypeDefinition,
};
use crate::instruction::regular_instruction::RegularInstruction;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for EntityTypeDefinition {
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::entity_type_definition(
                self.name.to_string(),
            ).into();

            // TODO: compile local impls etc...

            for item in self.definition.to_bytecode_items(visitor, cache) {
                yield item;
            }
        })
    }
}
