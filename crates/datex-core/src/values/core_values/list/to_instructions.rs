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
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        self.items.to_instructions(visitor, cache)
    }
}
