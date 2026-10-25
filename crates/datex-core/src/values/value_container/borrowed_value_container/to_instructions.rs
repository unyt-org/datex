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
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        match self {
            BorrowedValueContainer::Local(value) => {
                value.to_bytecode_items(visitor, cache)
            }
            BorrowedValueContainer::Shared(shared_container) => {
                shared_container.to_bytecode_items(visitor, cache)
            }
        }
    }
}
