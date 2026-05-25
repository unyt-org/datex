use crate::{
    dif::serde_context::SerdeContext,
    libs::core::{
        core_lib_id::{CoreLibId, CoreLibIdIndex},
    },
    types::type_definition::{TypeDefinition, tagged_type},
    utils::serde_serialize_seed::SerializeSeed,
};
use serde::{Serializer, ser::SerializeMap};
use crate::types::type_definition::list::ListTypeDefinition;
use crate::types::type_definition::map::MapTypeDefinition;
use crate::types::type_definition::range::RangeTypeDefinition;
use crate::utils::serde_serialize_seed::ValueWithSeed;

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
                    TypeDefinition::List(list_def) => {
                        outer.serialize_entry(value.as_ref(), &ValueWithSeed::new(
                            list_def,
                            self.cast::<ListTypeDefinition>(),
                        ))?
                    }
                    TypeDefinition::Map(map_def) => {
                        outer.serialize_entry(value.as_ref(), &ValueWithSeed::new(
                            map_def,
                            self.cast::<MapTypeDefinition>(),
                        ))?
                    }
                    TypeDefinition::Range(range) => {
                        outer.serialize_entry(value.as_ref(), &ValueWithSeed::new(
                            range,
                            self.cast::<RangeTypeDefinition>(),
                        ))?
                    }
                    TypeDefinition::Collection(collection_type_definition) => {
                        todo!()
                    }
                    TypeDefinition::Shared(
                        shared_container_containing_type,
                    ) => todo!(),
                    TypeDefinition::Nested(nested) => {
                        todo!()
                    }
                    TypeDefinition::Callable(callable_signature) => todo!(),
                    TypeDefinition::ImplType(def) => {
                        todo!()
                    }
                    TypeDefinition::Intersection(type_intersection) => {
                        todo!()
                    }
                    TypeDefinition::Union(type_union) => {
                        todo!()
                    }
                    TypeDefinition::TaggedType(tagged_type) => {
                        todo!()
                    }
                    TypeDefinition::Type => todo!(),
                    TypeDefinition::Core(_) => unreachable!(),
                }
                outer.end()
            }
        }
    }
}
