use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    global::stack_index::StackIndex,
    instruction::{Instruction, regular_instruction::RegularInstruction},
    runtime::cache::shared_references_cache::SharedReferencesCache,
    value_updates::update_data::IncrementUpdateData,
};

use crate::prelude::*;
impl ToInstructions for IncrementUpdateData {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a> {
        Box::new(gen {
            yield RegularInstruction::increment().into();
            yield_all!(self.value.to_instructions(visitor, cache));

            // target
            yield RegularInstruction::borrow_stack_value(StackIndex(0)).into();
        })
    }
}
