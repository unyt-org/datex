use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    values::value_container::borrowed_value_container::BorrowedValueContainer,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for BorrowedValueContainer<'_> {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        match self {
            BorrowedValueContainer::Local(value) => {
                value.to_instructions(visitor, cache)
            }
            BorrowedValueContainer::Shared(shared_container) => {
                shared_container.to_instructions(visitor, cache)
            }
        }
    }
}
