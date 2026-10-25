use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    types::type_definition_with_metadata::TypeMetadata,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for TypeMetadata {
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        todo!()
    }
}
