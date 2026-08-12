use core::ops::Deref;
use serde::{
    Deserializer, Serializer,
    de::DeserializeSeed,
    ser::{SerializeMap, SerializeSeq},
};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeTuple;
use crate::{
    dif::serde_context::SerdeContext,
    types::type_definition::callable::CallableTypeDefinition,
    utils::serde_serialize_seed::SerializeSeed,
};
use crate::types::r#type::Type;
use crate::types::type_definition::callable::CallableKind;
use crate::utils::serde_serialize_seed::ValueWithSeed;

impl<'ctx> SerializeSeed for SerdeContext<'ctx, CallableTypeDefinition> {
    type Value = CallableTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut obj = serializer.serialize_map(Some(1))?;
        obj.serialize_key("kind")?;
        obj.serialize_value(&value.kind)?;

        obj.serialize_key("requires_async")?;
        obj.serialize_value(&value.requires_async)?;

        obj.serialize_key("parameters")?;
        obj.serialize_value(&ValueWithSeed::new(
            &value.parameters,
            self.cast::<Vec<(Option<String>, Type)>>(),
        ))?;
        obj.serialize_key("rest_parameters")?;
        match &value.rest_parameter {
            Some((name, ty)) => {
                obj.serialize_value(&ValueWithSeed::new(
                    &(name.clone(), ty.deref().clone()),
                    self.cast::<(Option<String>, Type)>(),
                ))?;
            }
            None => {
                obj.serialize_value(&())?;
            }
        }

        obj.serialize_key("return_type")?;
        match &value.return_type {
            Some(return_type) => {
                obj.serialize_value(&ValueWithSeed::new(
                    return_type.deref(),
                    self.cast::<Type>(),
                ))?;
            }
            None => {
                obj.serialize_value(&())?;
            }
        }

        obj.serialize_key("yeet_type")?;
        match &value.yeet_type {
            Some(yeet_type) => {
                obj.serialize_value(&ValueWithSeed::new(
                    yeet_type.deref(),
                    self.cast::<Type>(),
                ))?;
            }
            None => {
                obj.serialize_value(&())?;
            }
        }

        obj.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Vec<(Option<String>, Type)>> {

    type Value = Vec<(Option<String>, Type)>;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for (name, ty) in value {
            seq.serialize_element(&ValueWithSeed::new(
                &(name.clone(), ty.clone()),
                self.cast::<(Option<String>, Type)>(),
            ))?;
        }
        seq.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, (Option<String>, Type)> {
    type Value = (Option<String>, Type);

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&value.0)?;
        tuple.serialize_element(&ValueWithSeed::new(
            &value.1,
            self.cast::<Type>(),
        ))?;
        tuple.end()
    }
}


impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, CallableTypeDefinition>
{
    type Value = CallableTypeDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}
impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, CallableTypeDefinition> {
    type Value = CallableTypeDefinition;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
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
                    parameters = Some(map.next_value_seed(self.cast::<Vec<(Option<String>, Type)>>())?);
                }
                "rest_parameters" => {
                    rest_parameter = map
                        .next_value_seed(self.cast::<Option<(Option<String>, Type)>>())?
                        .map(|(name, ty)| (name, Box::new(ty)));
                }
                "return_type" => {
                    return_type = map.next_value_seed(self.cast::<Option<Type>>())?;
                }
                "yeet_type" => {
                    yeet_type = map.next_value_seed(self.cast::<Option<Type>>())?;
                }
                _ => {
                    // Ignore unknown keys
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
        }

        let kind = kind.ok_or_else(|| serde::de::Error::missing_field("kind"))?;
        let parameters = parameters.ok_or_else(|| serde::de::Error::missing_field("parameters"))?;

        Ok(CallableTypeDefinition {
            kind,
            requires_async: false, // Default value, adjust as needed
            parameters,
            rest_parameter,
            return_type: return_type.map(Box::new),
            yeet_type: yeet_type.map(Box::new),
        })
    }
}

impl<'ctx, 'de> DeserializeSeed<'de> for SerdeContext<'ctx, (Option<String>, Type)> {
    type Value = (Option<String>, Type);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, (Option<String>, Type)> {
    type Value = (Option<String>, Type);

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
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

impl<'ctx, 'de> DeserializeSeed<'de> for SerdeContext<'ctx, Option<(Option<String>, Type)>> {
    type Value = Option<(Option<String>, Type)>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Option<(Option<String>, Type)>> {
    type Value = Option<(Option<String>, Type)>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
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
        let value = self.cast::<(Option<String>, Type)>().deserialize(deserializer)?;
        Ok(Some(value))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Vec<(Option<String>, Type)>> {
    type Value = Vec<(Option<String>, Type)>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Vec<(Option<String>, Type)>> {
    type Value = Vec<(Option<String>, Type)>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a sequence of (Option<String>, Type) tuples")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(item) = seq.next_element_seed(self.cast::<(Option<String>, Type)>())? {
            vec.push(item);
        }
        Ok(vec)
    }
}