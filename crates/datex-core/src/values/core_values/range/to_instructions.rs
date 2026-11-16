use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::core_values::range::Range,
};
use core::ops::Deref;

impl ToInstructions for Range {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::range().into();
            yield_all!(self.start.deref().to_instructions(visitor, cache));
            yield_all!(self.end.deref().to_instructions(visitor, cache));
        })
    }
}
