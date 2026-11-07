use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    instruction::{
        instruction_data::{
            CallableSignatureData, ImplMarkersData, IntersectionData, ListData,
            ListSliceCollectionData, MapData, ShortTextData, TaggedTypeData,
            UnionData,
        },
        type_instruction::TypeInstruction,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::{
        type_definition::{
            TypeDefinition,
            callable::CallableTypeDefinition,
            collection::{
                CollectionTypeDefinition,
                type_definition::{
                    ListCollectionTypeDefinition,
                    ListSliceCollectionTypeDefinition,
                    MapCollectionTypeDefinition,
                    ReceiverTypeDefinition,
                    SenderTypeDefinition,
                },
            },
            impl_type::ImplMarkers,
            intersection::IntersectionTypeDefinition,
            list::ListTypeDefinition,
            map::MapTypeDefinition,
            range::RangeTypeDefinition,
            tagged_type::TaggedTypeDefinition,
            union::UnionTypeDefinition,
        },
        type_definition_with_metadata::TypeDefinitionWithMetadata,
    },
};
use core::ops::Deref;

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
