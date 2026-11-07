use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition_with_metadata::TypeDefinitionWithMetadata,
};

impl ToInstructions for TypeDefinitionWithMetadata {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen {
            yield TypeInstruction::DefinitionWithMetadata(self.metadata).into();
            yield_all!(self.definition.to_instructions(visitor, cache))
        })
    }
}
