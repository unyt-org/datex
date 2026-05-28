use crate::{
    dif::serde_context::SerdeContext,
    libs::core::core_lib_id::{CoreLibId, CoreLibIdIndex},
    types::{
        collection_type_definition::CollectionTypeDefinition,
        r#type::Type,
        type_definition::{
            TypeDefinition,
            impl_type::ImplTypeDefinition,
            intersection::IntersectionTypeDefinition,
            list::ListTypeDefinition,
            map::MapTypeDefinition,
            range::RangeTypeDefinition,
            tagged_type::{self, TaggedTypeDefinition},
            union::UnionTypeDefinition,
        },
    },
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    values::core_values::callable::CallableSignature,
};
use serde::{Serializer, ser::SerializeMap};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, TypeDefinition> {
    type Value = TypeDefinition;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            TypeDefinition::Core(core) => serializer
                .serialize_u16(CoreLibIdIndex::from(CoreLibId::Type(*core)).0),
            _ => {
                let mut outer = serializer.serialize_map(Some(1))?;

                match value {
                    TypeDefinition::Literal(literal) => {
                        outer.serialize_entry(value.as_ref(), &literal)?;
                    }
                    TypeDefinition::List(list_def) => outer.serialize_entry(
                        value.as_ref(),
                        &ValueWithSeed::new(
                            list_def,
                            self.cast::<ListTypeDefinition>(),
                        ),
                    )?,
                    TypeDefinition::Map(map_def) => outer.serialize_entry(
                        value.as_ref(),
                        &ValueWithSeed::new(
                            map_def,
                            self.cast::<MapTypeDefinition>(),
                        ),
                    )?,
                    TypeDefinition::Range(range) => outer.serialize_entry(
                        value.as_ref(),
                        &ValueWithSeed::new(
                            range,
                            self.cast::<RangeTypeDefinition>(),
                        ),
                    )?,
                    TypeDefinition::Collection(collection_type_definition) => {
                        outer.serialize_entry(
                            value.as_ref(),
                            &ValueWithSeed::new(
                                collection_type_definition,
                                self.cast::<CollectionTypeDefinition>(),
                            ),
                        )?
                    }
                    TypeDefinition::Shared(
                        shared_container_containing_type,
                    ) => todo!(),
                    TypeDefinition::Nested(nested) => outer.serialize_entry(
                        value.as_ref(),
                        &ValueWithSeed::new(
                            nested as &Type,
                            self.cast::<Type>(),
                        ),
                    )?,
                    TypeDefinition::Callable(callable_signature) => outer
                        .serialize_entry(
                            value.as_ref(),
                            &ValueWithSeed::new(
                                callable_signature,
                                self.cast::<CallableSignature>(),
                            ),
                        )?,
                    TypeDefinition::ImplType(def) => outer.serialize_entry(
                        value.as_ref(),
                        &ValueWithSeed::new(
                            def,
                            self.cast::<ImplTypeDefinition>(),
                        ),
                    )?,
                    TypeDefinition::Intersection(type_intersection) => outer
                        .serialize_entry(
                            value.as_ref(),
                            &ValueWithSeed::new(
                                type_intersection,
                                self.cast::<IntersectionTypeDefinition>(),
                            ),
                        )?,
                    TypeDefinition::Union(type_union) => outer
                        .serialize_entry(
                            value.as_ref(),
                            &ValueWithSeed::new(
                                type_union,
                                self.cast::<UnionTypeDefinition>(),
                            ),
                        )?,
                    TypeDefinition::TaggedType(tagged_type) => outer
                        .serialize_entry(
                            value.as_ref(),
                            &ValueWithSeed::new(
                                tagged_type,
                                self.cast::<TaggedTypeDefinition>(),
                            ),
                        )?,
                    TypeDefinition::Type => todo!(),
                    TypeDefinition::Core(_) => unreachable!(), // already handled above
                }
                outer.end()
            }
        }
    }
}
