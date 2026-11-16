use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::core_values::{native::DatexNativeBase, receiver::Receiver},
};

impl<T> ToInstructions for Receiver<T>
where
    T: DatexNativeBase,
{
    fn to_instructions<'a>(
        &'a self,
        _visitor: &'a mut dyn ValueVisitor,
        _cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::Receiver.into();
        })
    }
}
