use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::collection::type_definition::MapCollectionTypeDefinition,
};
use core::ops::Deref;

impl ToInstructions for MapCollectionTypeDefinition {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::MapCollection.into();
            for instruction in self
                .key_type
                .deref()
                .to_instructions(visitor, cache)
                .collect::<Vec<_>>()
            {
                yield instruction;
            }
            for instruction in self
                .value_type
                .deref()
                .to_instructions(visitor, cache)
                .collect::<Vec<_>>()
            {
                yield instruction;
            }
        })
    }
}
