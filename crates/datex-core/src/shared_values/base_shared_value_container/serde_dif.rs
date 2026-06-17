use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    shared_values::base_shared_value_container::BaseSharedValueContainer,
    types::type_definition::TypeDefinition,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    values::value_container::ValueContainer,
};
use core::fmt;
use serde::{
    Deserializer, Serializer,
    de::{DeserializeSeed, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, BaseSharedValueContainer> {
    type Value = BaseSharedValueContainer;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // serialize as struct
        let mut state = serializer.serialize_seq(Some(3))?;

        state.serialize_element(&ValueWithSeed::new(
            &value.value_container,
            self.cast::<ValueContainer>(),
        ))?;

        state.serialize_element(&value.mutability)?;
        state.serialize_element(&ValueWithSeed::new(
            &value.allowed_type,
            self.cast::<TypeDefinition>(),
        ))?;
        state.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, BaseSharedValueContainer>
{
    type Value = BaseSharedValueContainer;

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, BaseSharedValueContainer> {
    type Value = BaseSharedValueContainer;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a valid BaseSharedValueContainer")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let value_container = seq
            .next_element_seed(self.cast::<ValueContainer>())?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

        let mutability = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

        let allowed_type =
            seq.next_element_seed(self.cast::<TypeDefinition>())?;

        Ok(if let Some(allowed_type) = allowed_type {
            BaseSharedValueContainer::try_new(
                value_container,
                allowed_type,
                mutability,
            )
            .map_err(|err| {
                serde::de::Error::custom(format!(
                    "invalid BaseSharedValueContainer: {err}"
                ))
            })?
        } else {
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                value_container,
                mutability,
            )
        })
    }
}
