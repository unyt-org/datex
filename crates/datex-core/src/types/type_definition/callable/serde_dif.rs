use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    types::{
        r#type::Type,
        type_definition::callable::{CallableKind, CallableTypeDefinition},
    },
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
};
use core::ops::Deref;
use serde::{
    Deserializer, Serializer,
    de::{DeserializeSeed, MapAccess, Visitor},
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
};
use crate::dif::serde_context::DeserializeSerdeContext;
use crate::utils::serde_serialize_seed::DeserializeWithSerdeContext;

impl<'ctx> SerializeSeed for CallableTypeDefinition {
    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut obj = serializer.serialize_map(Some(1))?;
        obj.serialize_key("kind")?;
        obj.serialize_value(&self.kind)?;

        obj.serialize_key("requires_async")?;
        obj.serialize_value(&self.requires_async)?;

        obj.serialize_key("parameters")?;
        obj.serialize_value(&ValueWithSerdeContext::new(
            &self.parameters,
            ctx,
        ))?;
        obj.serialize_key("rest_parameter")?;
        match &self.rest_parameter {
            Some((name, ty)) => {
                obj.serialize_value(&ValueWithSerdeContext::new(
                    &(name.clone(), ty.deref().clone()),
                    ctx,
                ))?;
            }
            None => {
                obj.serialize_value(&())?;
            }
        }

        obj.serialize_key("return_type")?;
        match &self.return_type {
            Some(return_type) => {
                obj.serialize_value(&ValueWithSerdeContext::new(
                    return_type.deref(),
                    ctx,
                ))?;
            }
            None => {
                obj.serialize_value(&())?;
            }
        }

        obj.serialize_key("yeet_type")?;
        match &self.yeet_type {
            Some(yeet_type) => {
                obj.serialize_value(&ValueWithSerdeContext::new(
                    yeet_type.deref(),
                    ctx,
                ))?;
            }
            None => {
                obj.serialize_value(&())?;
            }
        }

        obj.end()
    }
}

impl<'ctx> SerializeSeed for Vec<(Option<String>, Type)> {
    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for (name, ty) in self {
            seq.serialize_element(&ValueWithSerdeContext::new(
                &(name.clone(), ty.clone()),
                ctx,
            ))?;
        }
        seq.end()
    }
}

impl<'ctx> SerializeSeed for (Option<String>, Type) {

    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.0)?;
        tuple.serialize_element(&ValueWithSerdeContext::new(
            &self.1,
            ctx,
        ))?;
        tuple.end()
    }
}

impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for CallableTypeDefinition {
    fn deserialize_with_ctx<D>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(DeserializeSerdeContext::<CallableTypeDefinition>::new(ctx))
    }
}
impl<'de, 'a, 'ctx> Visitor<'de> for DeserializeSerdeContext<'a, 'ctx, CallableTypeDefinition> {
    type Value = CallableTypeDefinition;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a callable type definition")
    }

    fn visit_map<A>(mut self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut kind: Option<CallableKind> = None;
        let mut requires_async: Option<bool> = None;
        let mut parameters: Option<Vec<(Option<String>, Type)>> = None;
        let mut rest_parameter: Option<(Option<String>, Box<Type>)> = None;
        let mut return_type: Option<Type> = None;
        let mut yeet_type: Option<Type> = None;

        let mut map = map;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "kind" => {
                    kind = Some(map.next_value()?);
                }
                "requires_async" => {
                    requires_async = Some(map.next_value()?);
                }
                "parameters" => {
                    parameters = Some(map.next_value_seed(
                        self.cast::<Vec<(Option<String>, Type)>>(),
                    )?);
                }
                "rest_parameter" => {
                    rest_parameter = map
                        .next_value_seed(
                            self.cast::<Option<(Option<String>, Type)>>(),
                        )?
                        .map(|(name, ty)| (name, Box::new(ty)));
                }
                "return_type" => {
                    return_type =
                        map.next_value_seed(self.cast::<Option<Type>>())?;
                }
                "yeet_type" => {
                    yeet_type =
                        map.next_value_seed(self.cast::<Option<Type>>())?;
                }
                _ => {
                    // Ignore unknown keys
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
        }

        let kind =
            kind.ok_or_else(|| serde::de::Error::missing_field("kind"))?;
        let parameters = parameters
            .ok_or_else(|| serde::de::Error::missing_field("parameters"))?;

        Ok(CallableTypeDefinition {
            kind,
            requires_async: requires_async.unwrap_or(false),
            parameters,
            rest_parameter,
            return_type: return_type.map(Box::new),
            yeet_type: yeet_type.map(Box::new),
        })
    }
}

impl<'ctx, 'de> DeserializeWithSerdeContext<'de> for (Option<String>, Type) {

    fn deserialize_with_ctx<D>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, DeserializeSerdeContext::<(Option<String>, Type)>::new(ctx))
    }
}

impl<'de, 'a, 'ctx> Visitor<'de> for DeserializeSerdeContext<'a, 'ctx, (Option<String>, Type)> {
    type Value = (Option<String>, Type);

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a tuple of (Option<String>, Type)")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let name: Option<String> = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let ty: Type = seq
            .next_element_seed(self.cast::<Type>())?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        Ok((name, ty))
    }
}

impl<'ctx, 'de> DeserializeWithSerdeContext<'de> for Option<(Option<String>, Type)> {
    fn deserialize_with_ctx<D>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(DeserializeSerdeContext::<Option<(Option<String>, Type)>>::new(ctx))
    }
}

impl<'de, 'a, 'ctx> Visitor<'de>
    for DeserializeSerdeContext<'a, 'ctx, Option<(Option<String>, Type)>>
{
    type Value = Option<(Option<String>, Type)>;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("an optional tuple of (Option<String>, Type)")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = self
            .cast::<(Option<String>, Type)>()
            .deserialize(deserializer)?;
        Ok(Some(value))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }
}

impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for Vec<(Option<String>, Type)> {
    fn deserialize_with_ctx<D>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(DeserializeSerdeContext::<Vec<(Option<String>, Type)>>::new(ctx))
    }
}

impl<'de, 'a, 'ctx> Visitor<'de>
    for DeserializeSerdeContext<'a, 'ctx, Vec<(Option<String>, Type)>>
{
    type Value = Vec<(Option<String>, Type)>;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a sequence of (Option<String>, Type) tuples")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(item) =
            seq.next_element_seed(self.cast::<(Option<String>, Type)>())?
        {
            vec.push(item);
        }
        Ok(vec)
    }
}
