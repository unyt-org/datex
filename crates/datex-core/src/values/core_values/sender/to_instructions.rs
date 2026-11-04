use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::native::DatexNativeBase,
};
use crate::core_compiler::to_instructions::{yield_all, yield_all_collected};
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
            // Not allowed: local sender
            match self {
                Sender::LocalReceiver { .. } => {
                    panic!("Local sender cannot be converted to instructions")
                }
                Sender::RemoteReceiver { receiver } => {
                    // TODO: handle what happens when the receiver was dropped here - an unlinked sender must still be able to exist
                    yield_all_collected!(receiver.upgrade().unwrap().to_instructions(visitor, cache));
                }
            }
        })
    }
}
