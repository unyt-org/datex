use core::fmt;

use crate::{
    dif::serde_context::SerdeContext,
    libs::core::core_lib_id::{CoreLibId, CoreLibIdIndex},
    types::{
        literal_type_definition::LiteralTypeDefinition,
        r#type::Type,
        type_definition::{
            TypeDefinition, callable::CallableTypeDefinition,
            collection::CollectionTypeDefinition,
            impl_type::ImplTypeDefinition,
            intersection::IntersectionTypeDefinition, list::ListTypeDefinition,
            map::MapTypeDefinition, range::RangeTypeDefinition,
            tagged_type::TaggedTypeDefinition, union::UnionTypeDefinition,
        },
    },
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, Visitor},
    ser::SerializeMap,
};

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
                outer.serialize_key(value.as_ref())?;
                match value {
                    TypeDefinition::Literal(literal) => {
                        outer.serialize_value(&literal)?;
                    }
                    TypeDefinition::List(list_def) => {
                        outer.serialize_value(&ValueWithSeed::new(
                            list_def,
                            self.cast::<ListTypeDefinition>(),
                        ))?
                    }
                    TypeDefinition::Map(map_def) => {
                        outer.serialize_value(&ValueWithSeed::new(
                            map_def,
                            self.cast::<MapTypeDefinition>(),
                        ))?
                    }
                    TypeDefinition::Range(range) => {
                        outer.serialize_value(&ValueWithSeed::new(
                            range,
                            self.cast::<RangeTypeDefinition>(),
                        ))?
                    }
                    TypeDefinition::Collection(collection_type_definition) => {
                        outer.serialize_value(&ValueWithSeed::new(
                            collection_type_definition,
                            self.cast::<CollectionTypeDefinition>(),
                        ))?
                    }
                    TypeDefinition::Shared(
                        shared_container_containing_type,
                    ) => todo!(),
                    TypeDefinition::Nested(nested) => {
                        outer.serialize_value(&ValueWithSeed::new(
                            nested as &Type,
                            self.cast::<Type>(),
                        ))?
                    }
                    TypeDefinition::Callable(callable_signature) => outer
                        .serialize_value(&ValueWithSeed::new(
                            callable_signature,
                            self.cast::<CallableTypeDefinition>(),
                        ))?,
                    TypeDefinition::ImplType(def) => {
                        outer.serialize_value(&ValueWithSeed::new(
                            def,
                            self.cast::<ImplTypeDefinition>(),
                        ))?
                    }
                    TypeDefinition::Intersection(type_intersection) => outer
                        .serialize_value(&ValueWithSeed::new(
                            type_intersection,
                            self.cast::<IntersectionTypeDefinition>(),
                        ))?,
                    TypeDefinition::Union(type_union) => outer
                        .serialize_value(&ValueWithSeed::new(
                            type_union,
                            self.cast::<UnionTypeDefinition>(),
                        ))?,
                    TypeDefinition::TaggedType(tagged_type) => outer
                        .serialize_value(&ValueWithSeed::new(
                            tagged_type,
                            self.cast::<TaggedTypeDefinition>(),
                        ))?,
                    TypeDefinition::Type => outer.serialize_value("")?,
                    TypeDefinition::Core(_) => unreachable!(), // already handled above
                }
                outer.end()
            }
        }
    }
}

/// Deserialization for [TypeDefinition]
impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, TypeDefinition> {
    type Value = TypeDefinition;
    fn deserialize<D: Deserializer<'de>>(
        self,
        d: D,
    ) -> Result<TypeDefinition, D::Error> {
        d.deserialize_any(self)
    }
}
impl<'ctx> SerdeContext<'ctx, TypeDefinition> {
    fn deserialize_core_lib_id(
        &self,
        value: u64,
    ) -> Result<TypeDefinition, String> {
        let index = u16::try_from(value).map_err(|_| {
            format!(
                "CoreLibId index out of range for TypeDefinition: {}",
                value
            )
        })?;

        match CoreLibId::try_from(CoreLibIdIndex(index)) {
            Ok(CoreLibId::Type(core_type_id)) => {
                Ok(TypeDefinition::Core(core_type_id))
            }
            _ => Err(format!(
                "Invalid CoreLibId for TypeDefinition: {:?}",
                value
            )),
        }
    }
}
impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, TypeDefinition> {
    type Value = TypeDefinition;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a valid TypeDefinition")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let key: String = map.next_key()?.ok_or_else(|| {
            de::Error::custom("expected TypeDefinition map with one key")
        })?;

        let value = match key.as_str() {
            "literal" => {
                let literal = map.next_value()?;
                TypeDefinition::Literal(literal)
            }

            "list" => {
                let list =
                    map.next_value_seed(self.cast::<ListTypeDefinition>())?;
                TypeDefinition::List(list)
            }

            "map" => {
                let map_def =
                    map.next_value_seed(self.cast::<MapTypeDefinition>())?;
                TypeDefinition::Map(map_def)
            }

            "range" => {
                let range =
                    map.next_value_seed(self.cast::<RangeTypeDefinition>())?;
                TypeDefinition::Range(range)
            }

            "collection" => {
                let collection = map
                    .next_value_seed(self.cast::<CollectionTypeDefinition>())?;
                TypeDefinition::Collection(collection)
            }

            "nested" => {
                let ty = map.next_value_seed(self.cast::<Type>())?;
                TypeDefinition::Nested(Box::new(ty))
            }

            "callable" => {
                let callable =
                    map.next_value_seed(self.cast::<CallableTypeDefinition>())?;
                TypeDefinition::Callable(callable)
            }

            "impl_type" => {
                let def =
                    map.next_value_seed(self.cast::<ImplTypeDefinition>())?;
                TypeDefinition::ImplType(def)
            }

            "intersection" => {
                let intersection = map.next_value_seed(
                    self.cast::<IntersectionTypeDefinition>(),
                )?;
                TypeDefinition::Intersection(intersection)
            }

            "union" => {
                let union =
                    map.next_value_seed(self.cast::<UnionTypeDefinition>())?;
                TypeDefinition::Union(union)
            }

            "tagged_type" => {
                let tagged =
                    map.next_value_seed(self.cast::<TaggedTypeDefinition>())?;
                TypeDefinition::TaggedType(tagged)
            }

            "type" => {
                let _: String = map.next_value()?;
                TypeDefinition::Type
            }

            other => {
                return Err(de::Error::unknown_variant(
                    other,
                    &[
                        "literal",
                        "list",
                        "map",
                        "range",
                        "collection",
                        "nested",
                        "callable",
                        "impl_type",
                        "intersection",
                        "union",
                        "tagged_type",
                        "type",
                    ],
                ));
            }
        };

        if let Some(extra_key) = map.next_key::<String>()? {
            return Err(de::Error::custom(format!(
                "expected TypeDefinition map with exactly one key, found extra key `{}`",
                extra_key
            )));
        }

        Ok(value)
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.deserialize_core_lib_id(value as u64)
            .map_err(E::custom)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.deserialize_core_lib_id(value).map_err(E::custom)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value < 0 {
            return Err(E::custom(format!(
                "Invalid negative CoreLibId for TypeDefinition: {}",
                value
            )));
        }

        self.deserialize_core_lib_id(value as u64)
            .map_err(E::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dif::{cache::DIFSharedContainerCache, serde_context::SerdeContext},
        libs::core::{
            core_lib_id::CoreLibIdIndex,
            type_id::{CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId},
        },
        types::type_definition::TypeDefinition,
        values::core_values::integer::typed_integer::IntegerTypeVariant,
    };

    fn to_json(value: &TypeDefinition) -> String {
        SerdeContext::new(&mut DIFSharedContainerCache::default())
            .serialize_to_json(value)
    }
    use test_case::test_case;

    #[test_case(CoreLibTypeId::Base(CoreLibBaseTypeId::Text) ; "Text")]
    #[test_case(CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(IntegerTypeVariant::U8)) ; "integer/u8")]
    fn core_library_type_definition(id: CoreLibTypeId) {
        let type_def = TypeDefinition::Core(id);
        // Serialize the TypeDefinition to JSON
        let serialized = to_json(&type_def);

        // Assert that the serialized JSON is just the CoreLibId index as a number
        assert_eq!(serialized, format!(r#"{}"#, CoreLibIdIndex::from(id)));

        // Deserialize the JSON back to a TypeDefinition
        let deserialized: TypeDefinition =
            SerdeContext::new(&mut DIFSharedContainerCache::default())
                .try_deserialize_from_json(&serialized)
                .unwrap();
        assert_eq!(type_def, deserialized);
    }
}
