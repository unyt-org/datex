use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    values::value_container::ValueContainer,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for ValueContainer {
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        match self {
            ValueContainer::Local(value) => value.to_bytecode_items(visitor, cache),
            ValueContainer::Shared(shared) => {
                visitor.visit_shared_container(shared)
            }
        }
    }
}
