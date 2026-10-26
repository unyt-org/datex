use crate::core_compiler::to_instructions::{yield_all, ToInstructions};
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::BytecodeItem;
use crate::instruction::type_instruction::TypeInstruction;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::types::entity_type::EntityType;
use crate::prelude::*;

impl ToInstructions for EntityType {
    fn to_bytecode_items<'a>(
        &'a self, 
        visitor: &'a mut dyn ValueVisitor, 
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item=BytecodeItem> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::EntityType.into();
            yield_all!(self.0.to_bytecode_items(visitor, cache));
        })
    }
}