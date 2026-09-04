use crate::{
    core_compiler::to_instructions::{
        ToInstructions,
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
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;

impl<'ctx, T> ToInstructions<'ctx, T> for TypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        gen move {
            match self {
                TypeDefinition::ImplType(impl_type_def) => {
                    for instruction in impl_type_def.to_instructions(ctx) {
                        yield instruction;
                    }
                }
                TypeDefinition::Literal(literal_type_definition) => {
                    yield TypeInstruction::Literal(
                        literal_type_definition.clone(),
                    ).into();
                }
                TypeDefinition::List(list_type_definition) => {
                    for instruction in list_type_definition.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Map(map_type_definition) => {
                    for instruction in map_type_definition.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Range(range_type_definition) => {
                    for instruction in
                        range_type_definition.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Collection(collection_type_definition) => {
                    for instruction in
                        collection_type_definition.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Shared(shared_container_containing_type) => {
                    if let Some(tracking) = ctx.shared_value_tracking() {
                        tracking.borrow_mut().register_shared_value(
                            &shared_container_containing_type
                                .clone_with_move_indicator_if_owned(),
                        );
                    }
                }
                TypeDefinition::Box(nested) => {
                    yield TypeInstruction::Boxed.into();
                    for instruction in nested.to_instructions(ctx) {
                        yield instruction;
                    }
                }
                TypeDefinition::Callable(callable_type_definition) => {
                    for instruction in
                        callable_type_definition.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Intersection(intersection_type_definition) => {
                    for instruction in
                        intersection_type_definition.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::Union(union_type_definition) => {
                    for instruction in
                        union_type_definition.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::TaggedType(tagged_type_definition) => {
                    for instruction in
                        tagged_type_definition.to_instructions(ctx)
                    {
                        yield instruction;
                    }
                }
                TypeDefinition::CoreType(core_lib_type_id) => {
                    yield TypeInstruction::CoreType(*core_lib_type_id).into();
                }
            }
        }
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for TypeDefinitionWithMetadata
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen {
            yield TypeInstruction::DefinitionWithMetadata(self.metadata).into();
            for instruction in self.definition.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}
impl<'ctx, T> ToInstructions<'ctx, T> for ImplTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen {
            yield TypeInstruction::ImplType(ImplTypeData {
                impl_count: self.impl_markers.len() as u8,
                impls: self.impl_markers.to_vec(),
            }).into();
            for instruction in self.inner_type.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for ListTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen move {
            yield TypeInstruction::List(ListData {
                element_count: self.len() as u32,
            }).into();
            for ty in self.iter() {
                for instruction in ty.to_instructions(ctx).collect::<Vec<_>>() {
                    yield instruction;
                }
            }
        })
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for MapTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen move {
            yield TypeInstruction::Map(MapData {
                element_count: self.len() as u32,
            }).into();
            for (key_ty, value_ty) in self.iter() {
                // FIXME: collect required everywhere due to lifetime issues with the generator and optional mut ref
                for instruction in
                    key_ty.to_instructions(ctx).collect::<Vec<_>>()
                {
                    yield instruction;
                }
                for instruction in
                    value_ty.to_instructions(ctx).collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
        })
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for RangeTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen move {
            yield TypeInstruction::Range.into();
            for instruction in
                self.start.to_instructions(ctx).collect::<Vec<_>>()
            {
                yield instruction;
            }
            for instruction in self.end.to_instructions(ctx).collect::<Vec<_>>()
            {
                yield instruction;
            }
        })
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for CollectionTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen move {
            match self {
                CollectionTypeDefinition::List(list) => {
                    for instruction in list.to_instructions(ctx) {
                        yield instruction;
                    }
                }
                CollectionTypeDefinition::Map(map) => {
                    for instruction in map.to_instructions(ctx) {
                        yield instruction;
                    }
                }
                CollectionTypeDefinition::ListSlice(list) => {
                    for instruction in list.to_instructions(ctx) {
                        yield instruction;
                    }
                }
                CollectionTypeDefinition::Range(range) => {
                    for instruction in range.to_instructions(ctx) {
                        yield instruction;
                    }
                }
            }
        })
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for ListCollectionTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen {
            yield TypeInstruction::ListCollection.into();
            for instruction in self.0.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}
impl<'ctx, T> ToInstructions<'ctx, T> for MapCollectionTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen move {
            yield TypeInstruction::MapCollection.into();
            for instruction in
                self.key_type.to_instructions(ctx).collect::<Vec<_>>()
            {
                yield instruction;
            }
            for instruction in
                self.value_type.to_instructions(ctx).collect::<Vec<_>>()
            {
                yield instruction;
            }
        })
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for ListSliceCollectionTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen move {
            yield TypeInstruction::ListSliceCollection(
                ListSliceCollectionData {
                    element_count: self.size as u32,
                },
            ).into();
            for instruction in
                self.item_type.to_instructions(ctx).collect::<Vec<_>>()
            {
                yield instruction;
            }
        })
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for IntersectionTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen move {
            yield TypeInstruction::Intersection(IntersectionData {
                element_count: self.len() as u32,
            }).into();
            for ty in self.iter() {
                for instruction in ty.to_instructions(ctx).collect::<Vec<_>>() {
                    yield instruction;
                }
            }
        })
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for CallableTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
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
            }).into();
            for (_, ty) in &self.parameters {
                for instruction in ty.to_instructions(ctx).collect::<Vec<_>>() {
                    yield instruction;
                }
            }
            if let Some((_, rest_type)) = &self.rest_parameter {
                for instruction in
                    rest_type.to_instructions(ctx).collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
            if let Some(return_type) = &self.return_type {
                for instruction in
                    return_type.to_instructions(ctx).collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
            if let Some(yeet_type) = &self.yeet_type {
                for instruction in
                    yeet_type.to_instructions(ctx).collect::<Vec<_>>()
                {
                    yield instruction;
                }
            }
        })
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for UnionTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen move {
            yield TypeInstruction::Union(UnionData {
                element_count: self.len() as u32,
            }).into();
            for ty in self.iter() {
                for instruction in ty.to_instructions(ctx).collect::<Vec<_>>() {
                    yield instruction;
                }
            }
        })
    }
}

impl<'ctx, T> ToInstructions<'ctx, T> for TaggedTypeDefinition
where
    T: ValueVisitor<'ctx>,
{


    fn to_instructions<'a>(
        &'a self,
        ctx: &'a mut T,
    ) -> impl Iterator<Item = Instruction> + 'a where 'ctx: 'a {
        Box::new(gen move {
            yield TypeInstruction::TaggedType(TaggedTypeData::new(
                self.tag.clone(),
                self.ty.is_some(),
            )).into();
            if let Some(ty) = &self.ty {
                for instruction in ty.to_instructions(ctx).collect::<Vec<_>>() {
                    yield instruction;
                }
            }
        })
    }
}
