use crate::types::literal_type_definition::LiteralTypeDefinition;
use serde::{Serialize, Serializer};

impl Serialize for LiteralTypeDefinition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            LiteralTypeDefinition::Boolean(bool) => {
                serializer.serialize_bool(*bool)
            }
            LiteralTypeDefinition::Text(text) => {
                serializer.serialize_str(&text)
            }
            LiteralTypeDefinition::Integer(int) => int.serialize(serializer),
            LiteralTypeDefinition::Decimal(decimal) => {
                decimal.serialize(serializer)
            }
            LiteralTypeDefinition::TypedInteger(typed_integer) => {
                typed_integer.serialize(serializer)
            }
            LiteralTypeDefinition::TypedDecimal(typed_decimal) => {
                typed_decimal.serialize(serializer)
            }
            LiteralTypeDefinition::Endpoint(endpoint) => {
                endpoint.serialize(serializer)
            }
        }
    }
}
