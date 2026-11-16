use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    global::stack_index::StackIndex,
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    value_updates::update_data::AppendEntryUpdateData,
};

/// Appends an append entry operation on a shared container
impl ToInstructions for AppendEntryUpdateData {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen {
            yield RegularInstruction::append_entry().into();
            yield_all!(self.value.to_instructions(visitor, cache));

            // target
            yield RegularInstruction::borrow_stack_value(StackIndex(0)).into();
        })
    }
}
