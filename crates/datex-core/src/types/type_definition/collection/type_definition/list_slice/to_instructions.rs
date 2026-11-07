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
