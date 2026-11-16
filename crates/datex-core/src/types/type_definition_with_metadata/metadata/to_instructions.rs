use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition_with_metadata::TypeMetadata,
};

impl ToInstructions for TypeMetadata {
    fn to_instructions<'a>(
        &'a self,
        _visitor: &'a mut dyn ValueVisitor,
        _cache: &mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(core::iter::once(
            TypeInstruction::DefinitionWithMetadata(*self).into(),
        ))
    }
}
