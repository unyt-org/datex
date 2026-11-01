use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    shared_values::SharedContainer,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for SharedContainer {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        visitor.visit_shared_container(self, cache)
    }
}
