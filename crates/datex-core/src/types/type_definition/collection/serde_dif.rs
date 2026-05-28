use crate::{
    dif::serde_context::SerdeContext,
    types::{
        r#type::Type,
        type_definition::{
            collection::{
                CollectionTypeDefinition,
                type_definition::{
                    list::ListCollectionTypeDefinition,
                    list_slice::ListSliceCollectionTypeDefinition,
                    map::MapCollectionTypeDefinition,
                },
            },
            range::RangeTypeDefinition,
        },
    },
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use serde::{
    Serializer,
    ser::{SerializeMap, SerializeSeq},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, CollectionTypeDefinition> {
    type Value = CollectionTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut obj = serializer.serialize_map(Some(1))?;
        obj.serialize_key(value.as_ref())?;
        match value {
            CollectionTypeDefinition::List(iten) => {
                obj.serialize_value(&ValueWithSeed::new(
                    iten,
                    self.cast::<ListCollectionTypeDefinition>(),
                ))?
            }
            CollectionTypeDefinition::ListSlice(item) => {
                obj.serialize_value(&ValueWithSeed::new(
                    item,
                    self.cast::<ListSliceCollectionTypeDefinition>(),
                ))?
            }
            CollectionTypeDefinition::Map(map) => {
                obj.serialize_value(&ValueWithSeed::new(
                    map,
                    self.cast::<MapCollectionTypeDefinition>(),
                ))?
            }
            CollectionTypeDefinition::Range(range) => obj.serialize_value(
                &ValueWithSeed::new(range, self.cast::<RangeTypeDefinition>()),
            )?,
        }
        obj.end()
    }
}
