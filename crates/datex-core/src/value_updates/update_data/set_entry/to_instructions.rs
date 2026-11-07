use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    global::stack_index::StackIndex,
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    value_updates::update_data::SetEntryUpdateData,
};
/// Appends a set entry operation on a shared container
impl ToInstructions for SetEntryUpdateData {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a> {
        Box::new(gen {
            // key
            // FIXME can we reuse this and outsouce to set_entry into regular instructions?
            if let Some(string) = self.key.try_as_str() {
                yield RegularInstruction::set_entry_text(string.to_string())
                    .into();
            } else if let Some(index) = self.key.try_as::<u32>() {
                // TODO: integer conversion
                yield RegularInstruction::set_entry_index(*index).into();
            } else {
                yield RegularInstruction::set_entry_dynamic().into();
                yield_all!(self.key.to_instructions(visitor, cache));
            }

            // value
            yield_all!(self.value.to_instructions(visitor, cache));
            // target
            yield RegularInstruction::borrow_stack_value(StackIndex(0)).into();
        })
    }
}
