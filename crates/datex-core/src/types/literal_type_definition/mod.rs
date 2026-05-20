//! This module contains the implementation of the [LiteralTypeDefinition], which represents a type definition for a literal value, such as an integer, decimal, text, boolean, or endpoint.
use crate::{
    libs::core::type_id::{
        CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId,
    },
    prelude::*,
    types::{type_definition::TypeDefinition, type_match::TypeMatch},
    values::{
        core_values::{
            decimal::{Decimal, typed_decimal::TypedDecimal},
            endpoint::Endpoint,
            integer::{Integer, typed_integer::TypedInteger},
            text::Text,
        },
        value_container::ValueContainer,
    },
};
use core::{fmt::Display, hash::Hash};
pub mod equality;
pub mod type_match;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum LiteralTypeDefinition {
    Integer(Integer),
    TypedInteger(TypedInteger),
    Decimal(Decimal),
    TypedDecimal(TypedDecimal),
    Text(String), // TODO: use Text or String?
    Boolean(bool),
    Endpoint(Endpoint),
}

macro_rules! impl_from_typed_int {
    ($($t:ty),*) => {
        $(
            impl From<$t> for LiteralTypeDefinition {
                fn from(value: $t) -> Self {
                    LiteralTypeDefinition::TypedInteger(TypedInteger::from(value))
                }
            }
        )*
    }
}
impl_from_typed_int!(u8, u16, u32, u64, i8, i16, i32, i64);

impl From<String> for LiteralTypeDefinition {
    fn from(value: String) -> Self {
        LiteralTypeDefinition::Text(value)
    }
}
impl From<&str> for LiteralTypeDefinition {
    fn from(value: &str) -> Self {
        LiteralTypeDefinition::Text(value.to_string())
    }
}

impl From<Integer> for LiteralTypeDefinition {
    fn from(value: Integer) -> Self {
        LiteralTypeDefinition::Integer(value)
    }
}
impl From<TypedInteger> for LiteralTypeDefinition {
    fn from(value: TypedInteger) -> Self {
        LiteralTypeDefinition::TypedInteger(value)
    }
}

impl From<TypedDecimal> for LiteralTypeDefinition {
    fn from(value: TypedDecimal) -> Self {
        LiteralTypeDefinition::TypedDecimal(value)
    }
}

impl From<Decimal> for LiteralTypeDefinition {
    fn from(value: Decimal) -> Self {
        LiteralTypeDefinition::Decimal(value)
    }
}

impl From<Text> for LiteralTypeDefinition {
    fn from(value: Text) -> Self {
        LiteralTypeDefinition::Text(value.0)
    }
}
impl From<bool> for LiteralTypeDefinition {
    fn from(value: bool) -> Self {
        LiteralTypeDefinition::Boolean(value)
    }
}

impl From<Endpoint> for LiteralTypeDefinition {
    fn from(value: Endpoint) -> Self {
        LiteralTypeDefinition::Endpoint(value)
    }
}

impl LiteralTypeDefinition {
    /// Get the core lib type pointer id for this structural type definition
    pub fn core_lib_type_id(&self) -> CoreLibTypeId {
        match self {
            LiteralTypeDefinition::Integer(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Integer)
            }
            LiteralTypeDefinition::TypedInteger(typed) => {
                CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                    typed.variant(),
                ))
            }
            LiteralTypeDefinition::Decimal(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal)
            }
            LiteralTypeDefinition::TypedDecimal(typed) => {
                CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                    typed.variant(),
                ))
            }
            LiteralTypeDefinition::Text(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Text)
            }
            LiteralTypeDefinition::Boolean(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Boolean)
            }
            LiteralTypeDefinition::Endpoint(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Endpoint)
            }
        }
    }
}

impl Display for LiteralTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LiteralTypeDefinition::Integer(integer) => {
                core::write!(f, "{}", integer)
            }
            LiteralTypeDefinition::TypedInteger(typed_integer) => {
                core::write!(f, "{}", typed_integer)
            }
            LiteralTypeDefinition::Decimal(decimal) => {
                core::write!(f, "{}", decimal)
            }
            LiteralTypeDefinition::TypedDecimal(typed_decimal) => {
                core::write!(f, "{}", typed_decimal)
            }
            LiteralTypeDefinition::Text(text) => {
                core::write!(f, "{}", Text::from(text.to_string()))
            }
            LiteralTypeDefinition::Boolean(boolean) => {
                core::write!(f, "{}", boolean)
            }
            LiteralTypeDefinition::Endpoint(endpoint) => {
                core::write!(f, "{}", endpoint)
            }
        }
    }
}

impl From<LiteralTypeDefinition> for TypeDefinition {
    fn from(value: LiteralTypeDefinition) -> Self {
        TypeDefinition::Literal(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        types::{
            literal_type_definition::LiteralTypeDefinition, r#type::Type,
            type_definition::TypeDefinition, type_match::TypeMatch,
        },
        values::{
            core_value::CoreValue,
            core_values::{integer::Integer, text::Text},
            value_container::ValueContainer,
        },
    };

    #[test]
    fn structural_type_display() {
        let int_type = LiteralTypeDefinition::Integer(Integer::from(42));
        assert_eq!(int_type.to_string(), "42");

        let text_type = LiteralTypeDefinition::Text("Hello".to_string());
        assert_eq!(text_type.to_string(), r#""Hello""#);

        let list_type = TypeDefinition::List(vec![
            Type::from(LiteralTypeDefinition::Integer(Integer::from(1))),
            Type::from(LiteralTypeDefinition::Text("World".to_string())),
        ]);
        assert_eq!(list_type.to_string(), r#"[1, "World"]"#);

        let struct_type = TypeDefinition::Map(vec![
            (
                LiteralTypeDefinition::Text("id".to_string()).into(),
                int_type.into(),
            ),
            (
                LiteralTypeDefinition::Text("name".to_string()).into(),
                text_type.into(),
            ),
        ]);
        assert_eq!(struct_type.to_string(), r#"{"id": 42, "name": "Hello"}"#);
    }

    #[test]
    fn value_matching() {
        let int_type = LiteralTypeDefinition::Integer(Integer::from(42));
        let int_value =
            ValueContainer::from(CoreValue::Integer(Integer::from(42)));
        assert!(int_type.matched_by_value(&int_value));

        let text_type = LiteralTypeDefinition::Text("Hello".to_string());
        let text_value =
            ValueContainer::from(CoreValue::Text(Text::from("Hello")));
        assert!(text_type.matched_by_value(&text_value));
    }
}
