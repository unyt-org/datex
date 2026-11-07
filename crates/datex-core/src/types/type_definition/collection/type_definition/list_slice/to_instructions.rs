use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{
        instruction_data::ListSliceCollectionData,
        type_instruction::TypeInstruction,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::collection::type_definition::ListSliceCollectionTypeDefinition,
};
use core::ops::Deref;

impl ToInstructions for ListSliceCollectionTypeDefinition {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::ListSliceCollection(
                ListSliceCollectionData {
                    element_count: self.size as u32,
                },
            )
            .into();
            for instruction in self
                .item_type
                .deref()
                .to_instructions(visitor, cache)
                .collect::<Vec<_>>()
            {
                yield instruction;
            }
        })
    }
}
