use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    values::core_values::boolean::Boolean,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for Boolean {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        self.0.to_instructions(visitor, cache)
    }
}
