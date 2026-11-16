use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    global::stack_index::StackIndex,
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    value_updates::update_data::UpdateOperation,
};

/// Appends a single update operation on a shared container
impl ToInstructions for UpdateOperation {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a> {
        match self {
            UpdateOperation::SetEntry(data) => {
                (**data).to_instructions(visitor, cache)
            }
            UpdateOperation::AppendEntry(data) => {
                (**data).to_instructions(visitor, cache)
            }
            UpdateOperation::Replace(data) => {
                (**data).to_instructions(visitor, cache)
            }
            UpdateOperation::Clear => Box::new(gen move {
                yield RegularInstruction::clear().into();
                // target
                yield RegularInstruction::borrow_stack_value(StackIndex(0))
                    .into();
            }),
            UpdateOperation::DeleteEntry(data) => {
                (**data).to_instructions(visitor, cache)
            }
            UpdateOperation::ListSplice(data) => {
                (**data).to_instructions(visitor, cache)
            }
            UpdateOperation::Increment(data) => {
                (**data).to_instructions(visitor, cache)
            }
            UpdateOperation::Decrement(data) => {
                (**data).to_instructions(visitor, cache)
            }
        }
    }
}
