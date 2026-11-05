use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    instruction::{Instruction, type_instruction::TypeInstruction},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::entity_type::EntityType,
};

impl ToInstructions for EntityType {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::EntityType.into();
            yield_all!(self.0.to_instructions(visitor, cache));
        })
    }
}
