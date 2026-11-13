use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::core_values::native::DatexNativeBase,
};

impl<K> ToInstructions for Vec<K>
where
    K: DatexNativeBase,
{
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::list(self.len() as u32).into();
            for item in self.iter() {
                for instruction in item.to_instructions(visitor, cache) {
                    yield instruction;
                }
            }
        })
    }
}
