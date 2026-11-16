use crate::{
    core_compiler::{
        to_instructions::{ToInstructions, yield_all},
        value_visitor::ValueVisitor,
    },
    instruction::type_instruction::TypeInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::TypeDefinition,
};
use core::ops::Deref;

impl ToInstructions for TypeDefinition {
    fn to_instructions<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::Instruction> + 'a> {
        Box::new(gen move {
            match self {
                TypeDefinition::ImplMarkers(impl_type_def) => {
                    yield_all!(impl_type_def.to_instructions(visitor, cache));
                }
                TypeDefinition::Literal(literal_type_definition) => {
                    yield TypeInstruction::Literal(
                        literal_type_definition.clone(),
                    )
                    .into();
                }
                TypeDefinition::List(list_type_definition) => {
                    yield_all!(
                        list_type_definition.to_instructions(visitor, cache)
                    );
                }
                TypeDefinition::Map(map_type_definition) => {
                    yield_all!(
                        map_type_definition.to_instructions(visitor, cache)
                    );
                }
                TypeDefinition::Range(range_type_definition) => {
                    yield_all!(
                        range_type_definition.to_instructions(visitor, cache)
                    );
                }
                TypeDefinition::Collection(collection_type_definition) => {
                    yield_all!(
                        collection_type_definition
                            .to_instructions(visitor, cache)
                    );
                }
                TypeDefinition::Shared(shared_container_containing_type) => {
                    yield_all!(
                        shared_container_containing_type
                            .to_instructions(visitor, cache)
                    );
                }
                TypeDefinition::Box(nested) => {
                    yield TypeInstruction::Boxed.into();
                    yield_all!(nested.deref().to_instructions(visitor, cache));
                }
                TypeDefinition::Callable(callable_type_definition) => {
                    yield_all!(
                        callable_type_definition
                            .to_instructions(visitor, cache)
                    );
                }
                TypeDefinition::Intersection(intersection_type_definition) => {
                    yield_all!(
                        intersection_type_definition
                            .to_instructions(visitor, cache)
                    );
                }
                TypeDefinition::Union(union_type_definition) => {
                    yield_all!(
                        union_type_definition.to_instructions(visitor, cache)
                    );
                }
                TypeDefinition::TaggedType(tagged_type_definition) => {
                    yield_all!(
                        tagged_type_definition.to_instructions(visitor, cache)
                    );
                }
                TypeDefinition::CoreType(core_lib_type_id) => {
                    yield TypeInstruction::CoreType(*core_lib_type_id).into();
                }
            }
        })
    }
}
