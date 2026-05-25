use crate::{
    dif::serde_context::SerdeContext,
    libs::core::{
        core_lib_id::{CoreLibId, CoreLibIdIndex},
        type_id::CoreLibTypeId,
    },
    types::type_definition::{TypeDefinition, tagged_type},
    utils::serde_serialize_seed::SerializeSeed,
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
                let mut map = serializer.serialize_map(Some(1))?;

                match value {
                    TypeDefinition::Literal(literal) => {
                        map.serialize_entry(value.as_ref(), &literal)?;
                    }
                    TypeDefinition::List(list) => {
                        map.serialize_entry(value.as_ref(), &list)?
                    }
                    TypeDefinition::Map(map) => {
                        map.serialize_entry(value.as_ref(), &map)?
                    }
                    TypeDefinition::Range(range) => {
                        map.serialize_entry(value.as_ref(), &range)?
                    }
                    TypeDefinition::Collection(collection_type_definition) => {
                        map.serialize_entry(
                            value.as_ref(),
                            &collection_type_definition,
                        )?
                    }
                    TypeDefinition::Shared(
                        shared_container_containing_type,
                    ) => map.serialize_entry(
                        value.as_ref(),
                        &shared_container_containing_type,
                    )?,
                    TypeDefinition::Nested(nested) => {
                        map.serialize_entry(*value.as_ref(), &nested)?
                    }
                    TypeDefinition::Callable(callable_signature) => map
                        .serialize_entry(value.as_ref(), &callable_signature)?,
                    TypeDefinition::ImplType(def) => {
                        // FIXME
                        map.serialize_entry(value.as_ref(), &def)?
                    }
                    TypeDefinition::Intersection(type_intersection) => {
                        map.serialize_entry(value.as_ref(), &type_intersection)?
                    }
                    TypeDefinition::Union(type_union) => {
                        map.serialize_entry(value.as_ref(), &type_union)?
                    }
                    TypeDefinition::TaggedType(tagged_type) => {
                        map.serialize_entry(value.as_ref(), tagged_type)?
                    }
                    TypeDefinition::Type => todo!(),
                }
                map.end()
            }
        }
    }
}
