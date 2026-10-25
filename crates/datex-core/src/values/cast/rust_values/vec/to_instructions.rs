use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::native::DatexNativeBase,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl<K> ToInstructions for Vec<K>
where
    K: DatexNativeBase,
{
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::list(self.len() as u32).into();
            for item in self.iter() {
                for instruction in item.to_bytecode_items(visitor, cache) {
                    yield instruction;
                }
            }
        })
    }
}
