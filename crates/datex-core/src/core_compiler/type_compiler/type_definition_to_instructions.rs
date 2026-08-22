use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::ToInstructions,
    },
    instruction::{
        instruction_data::{
            CallableSignatureData, ImplTypeData, IntersectionData, ListData,
            ListSliceCollectionData, MapData, ShortTextData, TaggedTypeData,
            UnionData,
        },
        type_instruction::TypeInstruction,
    },
    prelude::*,
    types::{
        type_definition::{
            TypeDefinition,
            callable::CallableTypeDefinition,
            collection::{
                CollectionTypeDefinition,
                type_definition::{
                    list::ListCollectionTypeDefinition,
                    list_slice::ListSliceCollectionTypeDefinition,
                    map::MapCollectionTypeDefinition,
                },
            },
            impl_type::ImplTypeDefinition,
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

impl ToInstructions for TypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            match self {
                TypeDefinition::ImplType(impl_type_def) => {
                    for instruction in
                        impl_type_def.to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Literal(literal_type_definition) => {
                    yield TypeInstruction::Literal(
                        literal_type_definition.clone(),
                    );
                }
                TypeDefinition::List(list_type_definition) => {
                    for instruction in list_type_definition
                        .to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Map(map_type_definition) => {
                    for instruction in map_type_definition
                        .to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Range(range_type_definition) => {
                    for instruction in range_type_definition
                        .to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Collection(collection_type_definition) => {
                    for instruction in collection_type_definition
                        .to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Shared(shared_container_containing_type) => {
                    if let Some(shared_value_tracking) = shared_value_tracking {
                        (*shared_value_tracking).register_shared_value(
                            shared_container_containing_type.clone().into(),
                        );
                    }
                }
                TypeDefinition::Box(nested) => {
                    yield TypeInstruction::Boxed;
                    for instruction in
                        nested.to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Callable(callable_type_definition) => {
                    for instruction in callable_type_definition
                        .to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Intersection(intersection_type_definition) => {
                    for instruction in intersection_type_definition
                        .to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Union(union_type_definition) => {
                    for instruction in union_type_definition
                        .to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::TaggedType(tagged_type_definition) => {
                    for instruction in tagged_type_definition
                        .to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::CoreType(core_lib_type_id) => {
                    yield TypeInstruction::CoreType(*core_lib_type_id)
                }
            }
        })
    }
}

impl ToInstructions for TypeDefinitionWithMetadata {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            yield TypeInstruction::DefinitionWithMetadata(self.metadata);
            for instruction in
                self.definition.to_instructions(shared_value_tracking)
            {
                yield instruction;
            }
        })
    }
}
impl ToInstructions for ImplTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            yield TypeInstruction::ImplType(ImplTypeData {
                impl_count: self.impl_markers.len() as u8,
                impls: self.impl_markers.to_vec(),
            });
            for instruction in
                self.inner_type.to_instructions(shared_value_tracking)
            {
                yield instruction;
            }
        })
    }
}

impl ToInstructions for ListTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        mut shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::List(ListData {
                element_count: self.len() as u32,
            });
            for ty in self.iter() {
                for instruction in ty
                    .to_instructions(shared_value_tracking.as_deref_mut())
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
        })
    }
}

impl ToInstructions for MapTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        mut shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::Map(MapData {
                element_count: self.len() as u32,
            });
            for (key_ty, value_ty) in self.iter() {
                // FIXME: collect required everywhere due to lifetime issues with the generator and optional mut ref
                for instruction in key_ty
                    .to_instructions(shared_value_tracking.as_deref_mut())
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
                for instruction in value_ty
                    .to_instructions(shared_value_tracking.as_deref_mut())
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
        })
    }
}

impl ToInstructions for RangeTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        mut shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::Range;
            for instruction in self
                .start
                .to_instructions(shared_value_tracking.as_deref_mut())
                .collect::<Vec<_>>()
            {
                yield instruction;
            }
            for instruction in self
                .end
                .to_instructions(shared_value_tracking.as_deref_mut())
                .collect::<Vec<_>>()
            {
                yield instruction;
            }
        })
    }
}

impl ToInstructions for CollectionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            match self {
                CollectionTypeDefinition::List(list) => {
                    for instruction in
                        list.to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                CollectionTypeDefinition::Map(map) => {
                    for instruction in
                        map.to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                CollectionTypeDefinition::ListSlice(list) => {
                    for instruction in
                        list.to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
                CollectionTypeDefinition::Range(range) => {
                    for instruction in
                        range.to_instructions(shared_value_tracking)
                    {
                        yield instruction;
                    }
                }
            }
        })
    }
}

impl ToInstructions for ListCollectionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            yield TypeInstruction::ListCollection;
            for instruction in self.0.to_instructions(shared_value_tracking) {
                yield instruction;
            }
        })
    }
}
impl ToInstructions for MapCollectionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        mut shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::MapCollection;
            for instruction in self
                .key_type
                .to_instructions(shared_value_tracking.as_deref_mut())
                .collect::<Vec<_>>()
            {
                yield instruction;
            }
            for instruction in self
                .value_type
                .to_instructions(shared_value_tracking.as_deref_mut())
                .collect::<Vec<_>>()
            {
                yield instruction;
            }
        })
    }
}

impl ToInstructions for ListSliceCollectionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        mut shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::ListSliceCollection(
                ListSliceCollectionData {
                    element_count: self.size as u32,
                },
            );
            for instruction in self
                .item_type
                .to_instructions(shared_value_tracking.as_deref_mut())
                .collect::<Vec<_>>()
            {
                yield instruction;
            }
        })
    }
}

impl ToInstructions for IntersectionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        mut shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::Intersection(IntersectionData {
                element_count: self.len() as u32,
            });
            for ty in self.iter() {
                for instruction in ty
                    .to_instructions(shared_value_tracking.as_deref_mut())
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
        })
    }
}

impl ToInstructions for CallableTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        mut shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
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
            });
            for (_, ty) in &self.parameters {
                for instruction in ty
                    .to_instructions(shared_value_tracking.as_deref_mut())
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
            if let Some((_, rest_type)) = &self.rest_parameter {
                for instruction in rest_type
                    .to_instructions(shared_value_tracking.as_deref_mut())
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
            if let Some(return_type) = &self.return_type {
                for instruction in return_type
                    .to_instructions(shared_value_tracking.as_deref_mut())
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
            if let Some(yeet_type) = &self.yeet_type {
                for instruction in yeet_type
                    .to_instructions(shared_value_tracking.as_deref_mut())
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
        })
    }
}

impl ToInstructions for UnionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        mut shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::Union(UnionData {
                element_count: self.len() as u32,
            });
            for ty in self.iter() {
                for instruction in ty
                    .to_instructions(shared_value_tracking.as_deref_mut())
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
        })
    }
}

impl ToInstructions for TaggedTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions<'a>(
        &'a self,
        mut shared_value_tracking: Option<&'a mut SharedValueTracking>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen move {
            yield TypeInstruction::TaggedType(TaggedTypeData::new(
                self.tag.clone(),
                self.ty.is_some(),
            ));
            if let Some(ty) = &self.ty {
                for instruction in ty
                    .to_instructions(shared_value_tracking.as_deref_mut())
                    .collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
        })
    }
}
