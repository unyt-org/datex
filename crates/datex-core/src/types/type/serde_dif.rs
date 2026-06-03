use crate::{
    dif::serde_context::SerdeContext,
    libs::core::type_id::CoreLibBaseTypeId,
    shared_values::SharedContainer,
    types::{
        shared_container_containing_nominal_type::SharedContainerContainingNominalType,
        r#type::Type,
        type_definition::TypeDefinition,
        type_definition_with_metadata::{
            TypeDefinitionWithMetadata, TypeMetadata,
        },
    },
    utils::serde_serialize_seed::SerializeSeed,
};
use core::ops::Deref;
use num::ToPrimitive;
use serde::{
    Serializer,
    de::{DeserializeSeed, IntoDeserializer, Visitor},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Type> {
    type Value = Type;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Type::Alias(type_definition) => match type_definition.definition {
                TypeDefinition::CoreType(core)
                    if type_definition.metadata == TypeMetadata::default() =>
                {
                    self.cast::<TypeDefinition>()
                        .serialize(&type_definition.definition, serializer)
                }
                _ => self
                    .cast::<TypeDefinitionWithMetadata>()
                    .serialize(type_definition, serializer),
            },
            Type::Nominal(shared_container_containing_nominal_type) => {
                self.cast::<SharedContainer>().serialize(
                    shared_container_containing_nominal_type.deref(),
                    serializer,
                )
            }
        }
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Type> {
    type Value = Type;

    fn deserialize<D: serde::de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Type> {
    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a type definition")
    }
    type Value = Type;

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let metadata = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing metadata"))?;
        let definition = seq
            .next_element_seed(self.cast::<TypeDefinition>())?
            .ok_or_else(|| serde::de::Error::custom("missing definition"))?;
        Ok(Type::Alias(TypeDefinitionWithMetadata {
            metadata,
            definition,
        }))
    }

    fn visit_str<E>(mut self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Type::Nominal(unsafe {
            SharedContainerContainingNominalType::new_unchecked(
                self.cast::<SharedContainer>()
                    .deserialize(v.into_deserializer())?,
            )
        }))
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Type::Alias(
            TypeDefinition::core(CoreLibBaseTypeId::try_from(v).map_err(
                |_| {
                    serde::de::Error::custom(format!(
                        "invalid core type id: {v}"
                    ))
                },
            )?)
            .into(),
        ))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u16(v as u16)
    }
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u16(v.to_u16().ok_or_else(|| {
            serde::de::Error::custom(format!("core type id out of range: {v}"))
        })?)
    }
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u16(v.to_u16().ok_or_else(|| {
            serde::de::Error::custom(format!("core type id out of range: {v}"))
        })?)
    }
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u16(v as u16)
    }
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u16(v as u16)
    }
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u16(v as u16)
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u16(v as u16)
    }
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u16(v as u16)
    }
}
