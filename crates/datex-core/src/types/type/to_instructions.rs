use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::r#type::Type,
};

impl ToInstructions for Type {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        match self {
            Type::Entity(entity_type) => {
                entity_type.to_instructions(visitor, cache)
            }
            Type::Definition(def) => def.to_instructions(visitor, cache),
        }
    }
}
