use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::value_container::ValueContainer,
};

impl ToInstructions for ValueContainer {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        match self {
            ValueContainer::Local(value) => {
                value.to_instructions(visitor, cache)
            }
            ValueContainer::Shared(shared) => {
                visitor.visit_shared_container(shared, cache)
            }
        }
    }
}
