use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::shared_container_containing_type::SharedContainerContainingType,
};
use core::ops::Deref;

impl ToInstructions for SharedContainerContainingType {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        visitor.visit_shared_container(self.deref(), cache)
    }
}
