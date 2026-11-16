use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{
        instruction_data::TaggedTypeData, type_instruction::TypeInstruction,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::tagged_type::TaggedTypeDefinition,
};
use core::ops::Deref;

impl ToInstructions for TaggedTypeDefinition {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::TaggedType(TaggedTypeData::new(
                self.tag.clone(),
                self.ty.is_some(),
            ))
            .into();
            if let Some(ty) = &self.ty {
                for instruction in ty
                    .deref()
                    .to_instructions(visitor, cache)
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
        })
    }
}
