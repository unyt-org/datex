use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::native::DatexNativeBase,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::values::core_values::sender::Sender;

impl<K> ToInstructions for Sender<K>
where
    K: DatexNativeBase,
{
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::Sender.into();
        })
    }
}
