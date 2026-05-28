use crate::{
    libs::core::{
        core_lib_id::CoreLibId,
        type_id::{CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId},
    },
    prelude::*,
    types::literal_type_definition::LiteralTypeDefinition,
    values::core_values::{
        decimal::typed_decimal::TypedDecimal,
        integer::typed_integer::TypedInteger,
    },
};
use serde::{Serialize, Serializer, ser::SerializeMap};

impl Serialize for LiteralTypeDefinition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            LiteralTypeDefinition::Boolean(value) => {
                serializer.serialize_bool(*value)
            }

            LiteralTypeDefinition::Text(value) => {
                serializer.serialize_str(value)
            }

            LiteralTypeDefinition::Integer(value) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(
                    CoreLibBaseTypeId::Integer.to_string().as_str(),
                    value,
                )?;
                map.end()
            }

            LiteralTypeDefinition::Decimal(value) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(
                    CoreLibBaseTypeId::Decimal.to_string().as_str(),
                    value,
                )?;
                map.end()
            }

            LiteralTypeDefinition::TypedInteger(value) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(
                    &CoreLibId::Type(CoreLibTypeId::Variant(
                        CoreLibVariantTypeId::Integer(value.variant()),
                    ))
                    .to_string(),
                    value,
                )?;
                map.end()
            }

            LiteralTypeDefinition::TypedDecimal(value) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(
                    &CoreLibId::Type(CoreLibTypeId::Variant(
                        CoreLibVariantTypeId::Decimal(value.variant()),
                    ))
                    .to_string(),
                    value,
                )?;
                map.end()
            }

            LiteralTypeDefinition::Endpoint(value) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("endpoint", value)?;
                map.end()
            }
        }
    }
}

use core::fmt;
use serde::{
    Deserialize,
    de::{self, Deserializer, MapAccess, Visitor},
};

impl<'de> Deserialize<'de> for LiteralTypeDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(LiteralTypeDefinitionVisitor)
    }
}

struct LiteralTypeDefinitionVisitor;

impl<'de> Visitor<'de> for LiteralTypeDefinitionVisitor {
    type Value = LiteralTypeDefinition;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid LiteralTypeDefinition")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(LiteralTypeDefinition::Boolean(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(LiteralTypeDefinition::Text(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(LiteralTypeDefinition::Text(value))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let key: String = map
            .next_key()?
            .ok_or_else(|| de::Error::custom("expected literal type object"))?;

        let result = match key.as_str() {
            "endpoint" => {
                let value = map.next_value()?;
                LiteralTypeDefinition::Endpoint(value)
            }

            _ => {
                let type_id =
                    CoreLibTypeId::try_from_str(&key).ok_or_else(|| {
                        de::Error::custom(format!(
                            "invalid LiteralTypeDefinition type key `{}`",
                            key,
                        ))
                    })?;

                deserialize_literal_for_type_id(type_id, &mut map)?
            }
        };

        if map.next_key::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::custom(
                "expected LiteralTypeDefinition object with exactly one key",
            ));
        }

        Ok(result)
    }
}

fn deserialize_literal_for_type_id<'de, A>(
    type_id: CoreLibTypeId,
    map: &mut A,
) -> Result<LiteralTypeDefinition, A::Error>
where
    A: MapAccess<'de>,
{
    match type_id {
        CoreLibTypeId::Base(CoreLibBaseTypeId::Integer) => {
            let value = map.next_value()?;
            Ok(LiteralTypeDefinition::Integer(value))
        }

        CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal) => {
            let value = map.next_value()?;
            Ok(LiteralTypeDefinition::Decimal(value))
        }

        CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(variant)) => {
            let value: String = map.next_value()?;

            let value = TypedInteger::from_string_and_variant(&value, variant)
                .map_err(|err| {
                    de::Error::custom(format!(
                        "invalid typed integer literal for variant {:?}: {:?}",
                        variant, err
                    ))
                })?;

            Ok(LiteralTypeDefinition::TypedInteger(value))
        }

        CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(variant)) => {
            let value: String = map.next_value()?;

            let value = TypedDecimal::from_string_and_variant(&value, variant)
                .map_err(|err| {
                    de::Error::custom(format!(
                        "invalid typed decimal literal for variant {:?}: {:?}",
                        variant, err
                    ))
                })?;

            Ok(LiteralTypeDefinition::TypedDecimal(value))
        }

        other => Err(de::Error::custom(format!(
            "invalid LiteralTypeDefinition type key: {:?}",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use serde_json::{self, Value, json};
    use test_case::test_case;
    fn assert_json_string_roundtrip(value: LiteralTypeDefinition) {
        let serialized = serde_json::to_string(&value).unwrap();
        let deserialized: LiteralTypeDefinition =
            serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, value);
    }
    fn assert_json_roundtrip(value: LiteralTypeDefinition, expected: Value) {
        let serialized = serde_json::to_value(&value).unwrap();
        assert_eq!(serialized, expected);
        let deserialized: LiteralTypeDefinition =
            serde_json::from_value(serialized).unwrap();
        assert_eq!(deserialized, value);
    }

    #[test_case(LiteralTypeDefinition::Boolean(true), json!(true); "boolean_true")]
    #[test_case(LiteralTypeDefinition::Boolean(false), json!(false); "boolean_false")]
    #[test_case(LiteralTypeDefinition::Text("hello".to_string()), json!("hello"); "text")]
    #[test_case(LiteralTypeDefinition::Integer(123.into()), json!({
        CoreLibTypeId::Base(CoreLibBaseTypeId::Integer).to_string(): "123"
    }); "integer_positive")]
    #[test_case(LiteralTypeDefinition::Integer((-123).into()), json!({
        CoreLibTypeId::Base(CoreLibBaseTypeId::Integer).to_string(): "-123"
    }); "integer_negative")]
    #[test_case(LiteralTypeDefinition::Integer(0.into()), json!({
        CoreLibTypeId::Base(CoreLibBaseTypeId::Integer).to_string(): "0"
    }); "integer_zero")]
    // FIXME implement serialize for decimals correct
    // #[test_case(LiteralTypeDefinition::Decimal(123.0.into()), json!({
    //     CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal).to_string(): "123.0"
    // }); "decimal_positive")]
    // #[test_case(LiteralTypeDefinition::Decimal((-123.0).into()), json!({
    //     CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal).to_string(): "-123.0"
    // }); "decimal_negative")]
    // #[test_case(LiteralTypeDefinition::Decimal(0.0.into()), json!({
    //     CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal).to_string(): "0.0"
    // }); "decimal_zero")]
    fn literal_type_definition_roundtrips(
        value: LiteralTypeDefinition,
        expected: Value,
    ) {
        assert_json_roundtrip(value, expected);
    }
}
