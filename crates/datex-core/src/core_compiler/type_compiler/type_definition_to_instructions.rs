use crate::{
    core_compiler::{
        shared_value_tracking::SharedValueTracking,
        to_instructions::ToInstructions,
    },
    global::protocol_structures::{
        instruction_data::{ImplTypeData, ListData, MapData},
        type_instructions::TypeInstruction,
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

impl<'a> ToInstructions<'a> for TypeDefinitionWithMetadata {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            yield TypeInstruction::TypeDefinitionWithMetadata(self.metadata);
            for instruction in
                self.definition.to_instructions(shared_value_tracking)
            {
                yield instruction;
            }
        })
    }
}

impl<'a> ToInstructions<'a> for TypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
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
                    yield TypeInstruction::TypeDefinitionLiteral(
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
                    let instructions: Vec<_> = shared_container_containing_type
                        .with_collapsed_type_value(|f| {
                            f.to_instructions(shared_value_tracking).collect()
                        });
                    for instruction in instructions {
                        yield instruction;
                    }
                }
                TypeDefinition::Nested(nested) => {
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
                    yield TypeInstruction::TypeDefinitionCoreType(
                        *core_lib_type_id,
                    )
                }
            }
        })
    }
}

impl<'a> ToInstructions<'a> for ImplTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            yield TypeInstruction::TypeDefinitionImplType(ImplTypeData {
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

impl<'a> ToInstructions<'a> for ListTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            yield TypeInstruction::TypeDefinitionList(ListData {
                element_count: self.len() as u32,
            });
            for ty in self.iter() {
                for instruction in ty.to_instructions(shared_value_tracking) {
                    yield instruction;
                }
            }
        })
    }
}

impl<'a> ToInstructions<'a> for MapTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            yield TypeInstruction::TypeDefinitionMap(MapData {
                element_count: self.len() as u32,
            });
            for (key_ty, value_ty) in self.iter() {
                for instruction in key_ty.to_instructions(shared_value_tracking)
                {
                    yield instruction;
                }
                for instruction in
                    value_ty.to_instructions(shared_value_tracking)
                {
                    yield instruction;
                }
            }
        })
    }
}

impl<'a> ToInstructions<'a> for RangeTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            yield TypeInstruction::TypeDefinitionRange;
            for instruction in self.start.to_instructions(shared_value_tracking)
            {
                yield instruction;
            }
            for instruction in self.end.to_instructions(shared_value_tracking) {
                yield instruction;
            }
        })
    }
}

impl<'a> ToInstructions<'a> for CollectionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
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

impl<'a> ToInstructions<'a> for ListCollectionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            for instruction in self.0.to_instructions(shared_value_tracking) {
                yield instruction;
            }
        })
    }
}
impl<'a> ToInstructions<'a> for MapCollectionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            for instruction in
                self.key_type.to_instructions(shared_value_tracking)
            {
                yield instruction;
            }
            for instruction in
                self.value_type.to_instructions(shared_value_tracking)
            {
                yield instruction;
            }
        })
    }
}

impl<'a> ToInstructions<'a> for ListSliceCollectionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        _shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen { todo!() })
    }
}

impl<'a> ToInstructions<'a> for IntersectionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            for ty in self.iter() {
                for instruction in ty.to_instructions(shared_value_tracking) {
                    yield instruction;
                }
            }
        })
    }
}

impl<'a> ToInstructions<'a> for CallableTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        _shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen { todo!() })
    }
}

impl<'a> ToInstructions<'a> for UnionTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen {
            for ty in self.iter() {
                for instruction in ty.to_instructions(shared_value_tracking) {
                    yield instruction;
                }
            }
        })
    }
}

impl<'a> ToInstructions<'a> for TaggedTypeDefinition {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &'a self,
        _shared_value_tracking: &'a mut SharedValueTracking,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'a> {
        Box::new(gen { todo!() })
    }
}
