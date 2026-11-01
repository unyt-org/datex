use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    types::r#type::Type,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for Type {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            match self {
                Type::Entity(entity_type) =>yield_all!(entity_type.to_instructions(visitor, cache)),
                Type::Definition(def) => {
                    yield_all!(def.to_instructions(visitor, cache));
                }
            }
        })
    }
}
