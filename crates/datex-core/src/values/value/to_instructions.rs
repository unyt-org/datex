use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    values::value::Value,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for Value {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            // append inner instructions
            match self {
                Value::Native(native) => {
                    yield_all!(native.to_instructions(visitor, cache));
                }
                Value::Core(core) => {
                    yield_all!(core.to_instructions(visitor, cache));
                }
            }
        })
    }
}
