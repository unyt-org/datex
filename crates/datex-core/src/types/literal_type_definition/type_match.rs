use crate::{
    libs::core::type_id::{
        CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId,
    },
    types::{
        literal_type_definition::LiteralTypeDefinition, type_match::TypeMatch,
    },
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
impl TypeMatch for LiteralTypeDefinition {
    fn matches(&self, other: &Self) -> bool {
        self == other
    }
    fn matched_by_value(&self, value: &ValueContainer) -> bool {
        match self {
            LiteralTypeDefinition::Integer(expected) => {
                value.try_as().map(|v: Integer| v == *expected)
            }
            LiteralTypeDefinition::TypedInteger(expected) => {
                value.try_as().map(|v: TypedInteger| v == *expected)
            }
            LiteralTypeDefinition::Decimal(expected) => {
                value.try_as().map(|v: Decimal| v == *expected)
            }
            LiteralTypeDefinition::TypedDecimal(expected) => {
                value.try_as().map(|v: TypedDecimal| v == *expected)
            }
            LiteralTypeDefinition::Text(expected) => {
                value.try_as().map(|v: Text| v.0 == *expected)
            }
            LiteralTypeDefinition::Boolean(expected) => {
                value.try_as().map(|v: bool| v == *expected)
            }
            LiteralTypeDefinition::Endpoint(expected) => {
                value.try_as().map(|v: Endpoint| v == *expected)
            }
        }
        .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        types::{
            literal_type_definition::LiteralTypeDefinition,
            type_match::TypeMatch,
        },
        values::core_values::integer::Integer,
    };

    #[test]
    fn integer() {
        let integer = LiteralTypeDefinition::Integer(Integer::new(42));
        assert!(integer.matched_by_value(&Integer::new(42).into()));

        let integer_u8 = LiteralTypeDefinition::TypedInteger(42u8.into());
        assert!(integer_u8.matched_by_value(&42u8.into()));

        let integer_wrong = LiteralTypeDefinition::Integer(Integer::new(43));
        assert!(!integer_wrong.matched_by_value(&Integer::new(42).into()));
    }
}
