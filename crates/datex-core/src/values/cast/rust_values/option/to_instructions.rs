use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl<K> ToInstructions for Option<K>
where
    K: ToInstructions,
{
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        Box::new(gen move {
            match self {
                None => yield RegularInstruction::Null.into(),
                Some(value) => {
                    for instruction in value.to_bytecode_items(visitor, cache) {
                        yield instruction;
                    }
                }
            }
        })
    }
}
