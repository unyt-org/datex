use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all, yield_all_collected},
        value_visitor::ValueVisitor,
    },
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};

impl<K, V, S> ToInstructions for indexmap::IndexMap<K, V, S>
where
    K: ToInstructions,
    V: ToInstructions,
{
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::map(self.len() as u32, false).into();

            for (key, value) in self.iter() {
                yield RegularInstruction::key_value_dynamic().into();
                yield_all_collected!(key.to_instructions(visitor, cache));
                yield_all!(value.to_instructions(visitor, cache));
            }
        })
    }
}
