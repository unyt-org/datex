use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    global::stack_index::StackIndex,
    instruction::regular_instruction::RegularInstruction,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    value_updates::update_data::ListSpliceUpdateData,
};

use crate::prelude::*;
/// Appends a list splice operation on a shared container
impl ToInstructions for ListSpliceUpdateData {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen {
            yield RegularInstruction::splice(
                self.start,
                self.delete_count,
                self.items.len() as u32,
            )
            .into();

            for item in &self.items {
                yield_all!(item.to_instructions(visitor, cache));
            }

            // target
            yield RegularInstruction::borrow_stack_value(StackIndex(0)).into();
        })
    }
}
