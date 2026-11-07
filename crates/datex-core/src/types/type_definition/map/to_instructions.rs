use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    instruction::{
        instruction_data::MapData, type_instruction::TypeInstruction,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::map::MapTypeDefinition,
};

impl ToInstructions for MapTypeDefinition {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::Map(MapData {
                element_count: self.len() as u32,
                allow_only_text_keys: false,
            })
            .into();
            for (key_ty, value_ty) in self.iter() {
                yield_all!(key_ty.to_instructions(visitor, cache));
                yield_all!(value_ty.to_instructions(visitor, cache));
            }
        })
    }
}
