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
                    MapCollectionTypeDefinition, ReceiverTypeDefinition,
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

impl ToInstructions for CallableTypeDefinition {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::Callable(CallableSignatureData {
                name: ShortTextData::new("".to_string()),
                kind: self.kind,
                parameter_count: self.parameters.len() as u8,
                requires_async: self.requires_async,
                has_rest_parameter: self.rest_parameter.is_some(),
                has_return_type: self.return_type.is_some(),
                has_yeet_type: self.yeet_type.is_some(),
                parameter_names: self
                    .parameters
                    .iter()
                    .map(|(name, _)| {
                        ShortTextData::new(name.clone().unwrap_or_default())
                    })
                    .collect(),
                rest_parameter_name: self.rest_parameter.as_ref().map(
                    |(name, _)| {
                        ShortTextData::new(name.clone().unwrap_or_default())
                    },
                ),
            })
            .into();
            for (_, ty) in &self.parameters {
                for instruction in
                    ty.to_instructions(visitor, cache).collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
            if let Some((_, rest_type)) = &self.rest_parameter {
                for instruction in rest_type
                    .deref()
                    .to_instructions(visitor, cache)
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
            if let Some(return_type) = &self.return_type {
                for instruction in return_type
                    .deref()
                    .to_instructions(visitor, cache)
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
            if let Some(yeet_type) = &self.yeet_type {
                for instruction in yeet_type
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
