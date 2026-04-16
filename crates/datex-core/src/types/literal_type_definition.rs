use crate::{
    libs::core::CoreLibTypeId,
    prelude::*,
    traits::structural_eq::StructuralEq,
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean,
            decimal::{typed_decimal::TypedDecimal, Decimal},
            endpoint::Endpoint,
            integer::{typed_integer::TypedInteger, Integer},
            text::Text,
        },
        value_container::ValueContainer,
    },
};
use core::{fmt::Display, hash::Hash, unimplemented};
use crate::types::r#type::Type;
use crate::types::structural_type_definition::StructuralTypeDefinition;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum LiteralTypeDefinition {
    Integer(Integer),
    TypedInteger(TypedInteger),
    Decimal(Decimal),
    TypedDecimal(TypedDecimal),
    Text(String),
    Boolean(bool),
    Endpoint(Endpoint),
    Null,
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
    /// Matches a value against self
    /// Returns true if all possible realizations of the value match the type
    /// Examples:
    /// 1 matches 1 -> true
    /// 1 matches 2 -> false
    /// 1 matches 1 | 2 -> true
    /// 1 | 2 matches integer -> true
    /// integer matches 1 | 2 -> false
    pub fn value_matches(&self, value: &ValueContainer) -> bool {
        match (self, &value.to_cloned_value().borrow().inner) {
            (LiteralTypeDefinition::Integer(a), CoreValue::Integer(b)) => {
                a == b
            }
            (
                LiteralTypeDefinition::TypedInteger(a),
                CoreValue::TypedInteger(b),
            ) => a == b,
            (LiteralTypeDefinition::Decimal(a), CoreValue::Decimal(b)) => {
                a == b
            }
            (
                LiteralTypeDefinition::TypedDecimal(a),
                CoreValue::TypedDecimal(b),
            ) => a == b,
            (LiteralTypeDefinition::Text(a), CoreValue::Text(b)) => a == b,
            (LiteralTypeDefinition::Boolean(a), CoreValue::Boolean(b)) => {
                a == b
            }
            (LiteralTypeDefinition::Endpoint(a), CoreValue::Endpoint(b)) => {
                a == b
            }
            (LiteralTypeDefinition::Null, CoreValue::Null) => true,

            // // Check that all elements in the list match the element type
            // (
            //     StructuralTypeDefinition::List(box elem_type),
            //     CoreValue::List(list),
            // ) => list.into_iter().all(|item| elem_type.value_matches(item)),
            //
            // // Check that all keys and values in the map match their types
            // (
            //     StructuralTypeDefinition::Map(box (key_type, value_type)),
            //     CoreValue::Map(map),
            // ) => map.iter().all(|(k, v)| {
            //     key_type.value_matches(k) && value_type.value_matches(v)
            // }),

            // Check that all fields in the map are present and match their types
            (
                LiteralTypeDefinition::Map(field_types),
                CoreValue::Map(_map),
            ) => field_types.iter().all(|(_field_name, _field_type)| {
                core::todo!("#375 handle key matching")
                // map.get(&field_name_value).is_some_and(|field_value| {
                //     field_type.value_matches(field_value)
                // })
            }),

            // list
            (
                LiteralTypeDefinition::List(type_list),
                CoreValue::List(list),
            ) => {
                if type_list.len() != list.len() as usize {
                    return false;
                }
                type_list
                    .iter()
                    .zip(list.iter())
                    .all(|(t, v)| t.value_matches(v))
            }
            _ => unimplemented!("handle complex structural type matching"),
        }
    }

    /// Get the core lib type pointer id for this structural type definition
    pub fn get_core_lib_type_pointer_id(&self) -> CoreLibTypeId {
        match self {
            LiteralTypeDefinition::Integer(_) => {
                CoreLibTypeId::Integer(None)
            }
            LiteralTypeDefinition::TypedInteger(typed) => {
                CoreLibTypeId::Integer(Some(typed.variant()))
            }
            LiteralTypeDefinition::Decimal(_) => {
                CoreLibTypeId::Decimal(None)
            }
            LiteralTypeDefinition::TypedDecimal(typed) => {
                CoreLibTypeId::Decimal(Some(typed.variant()))
            }
            LiteralTypeDefinition::Text(_) => CoreLibTypeId::Text,
            LiteralTypeDefinition::Boolean(_) => CoreLibTypeId::Boolean,
            LiteralTypeDefinition::Endpoint(_) => CoreLibTypeId::Endpoint,
            LiteralTypeDefinition::Null => CoreLibTypeId::Null,
            LiteralTypeDefinition::List(_) => CoreLibTypeId::List,
            LiteralTypeDefinition::Range(_) => CoreLibTypeId::Range,
            LiteralTypeDefinition::Map(_) => CoreLibTypeId::Map,
        }
    }
}

impl StructuralEq for LiteralTypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
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
            LiteralTypeDefinition::Text(text) => core::write!(f, "{}", text),
            LiteralTypeDefinition::Boolean(boolean) => {
                core::write!(f, "{}", boolean)
            }
            LiteralTypeDefinition::Endpoint(endpoint) => {
                core::write!(f, "{}", endpoint)
            }
            LiteralTypeDefinition::Null => core::write!(f, "null"),
            LiteralTypeDefinition::Range((start, end)) => {
                core::write!(f, "{}..{}", start, end)
            }
            LiteralTypeDefinition::List(types) => {
                let types_str: Vec<String> =
                    types.iter().map(|t| t.to_string()).collect();
                core::write!(f, "[{}]", types_str.join(", "))
            }
            LiteralTypeDefinition::Map(fields) => {
                let fields_str: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                core::write!(f, "{{{}}}", fields_str.join(", "))
            }
        }
    }
}

impl From<LiteralTypeDefinition> for StructuralTypeDefinition {
    fn from(value: LiteralTypeDefinition) -> Self {
        StructuralTypeDefinition::Literal(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        types::literal_type_definition::LiteralTypeDefinition,
        values::{
            core_value::CoreValue,
            core_values::{
                integer::Integer,
                text::Text,
            },
            value_container::ValueContainer,
        },
    };
    use crate::types::r#type::{Type, TypeMetadata};

    #[test]
    fn test_structural_type_display() {
        let int_type = LiteralTypeDefinition::Integer(Integer::from(42));
        assert_eq!(int_type.to_string(), "42");

        let text_type = LiteralTypeDefinition::Text(Text::from("Hello"));
        assert_eq!(text_type.to_string(), r#""Hello""#);

        let list_type = LiteralTypeDefinition::List(vec![
            Type::structural(
                LiteralTypeDefinition::Integer(Integer::from(1)),
                TypeMetadata::default(),
            )
            .into(),
            Type::structural(
                LiteralTypeDefinition::Text(Text::from("World")),
                TypeMetadata::default(),
            )
            .into(),
        ]);
        assert_eq!(list_type.to_string(), r#"[1, "World"]"#);

        let struct_type = LiteralTypeDefinition::Map(vec![
            (
                Type::structural("id".to_string(), TypeMetadata::default())
                    .into(),
                Type::structural(int_type.clone(), TypeMetadata::default())
                    .into(),
            ),
            (
                Type::structural("name".to_string(), TypeMetadata::default())
                    .into(),
                Type::structural(text_type.clone(), TypeMetadata::default())
                    .into(),
            ),
        ]);
        assert_eq!(struct_type.to_string(), r#"{"id": 42, "name": "Hello"}"#);
    }

    #[test]
    fn test_value_matching() {
        let int_type = LiteralTypeDefinition::Integer(Integer::from(42));
        let int_value =
            ValueContainer::from(CoreValue::Integer(Integer::from(42)));
        assert!(int_type.value_matches(&int_value));

        let text_type = LiteralTypeDefinition::Text(Text::from("Hello"));
        let text_value =
            ValueContainer::from(CoreValue::Text(Text::from("Hello")));
        assert!(text_type.value_matches(&text_value));
    }
}
