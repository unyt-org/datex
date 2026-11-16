use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{
        instruction_data::ImplMarkersData, type_instruction::TypeInstruction,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::impl_type::ImplMarkers,
};

impl ToInstructions for ImplMarkers {
    fn to_instructions<'a>(
        &'a self,
        _visitor: &'a mut dyn ValueVisitor,
        _cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen {
            yield TypeInstruction::ImplMarkers(ImplMarkersData {
                impl_count: self.impl_markers.len() as u8,
                impls: self.impl_markers.to_vec(),
            })
            .into();
        })
    }
}
