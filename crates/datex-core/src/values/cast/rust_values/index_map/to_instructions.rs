use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    prelude::*,
};
use crate::core_compiler::to_instructions::{yield_all, yield_all_collected};
use crate::instruction::regular_instruction::RegularInstruction;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl<K, V, S> ToInstructions for indexmap::IndexMap<K, V, S>
where
    K: ToInstructions,
    V: ToInstructions,
{
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        Box::new(gen move {
            yield RegularInstruction::map(self.len() as u32).into();

            for (key, value) in self.iter() {
                yield RegularInstruction::key_value_dynamic().into();
                yield_all_collected!(key.to_bytecode_items(visitor, cache));
                yield_all!(value.to_bytecode_items(visitor, cache));
            }
        })
    }
}
