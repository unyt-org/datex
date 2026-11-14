use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{
        instruction_data::IntersectionData, type_instruction::TypeInstruction,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::intersection::IntersectionTypeDefinition,
};

impl ToInstructions for IntersectionTypeDefinition {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::Intersection(IntersectionData {
                element_count: self.len() as u32,
            })
            .into();
            for ty in self.iter() {
                for instruction in
                    ty.to_instructions(visitor, cache).collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
        })
    }
}
