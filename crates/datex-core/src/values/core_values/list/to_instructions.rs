use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    values::core_values::list::List,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for List {
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        self.items.to_bytecode_items(visitor, cache)
    }
}
