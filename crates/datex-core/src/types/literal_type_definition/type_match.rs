use crate::{
    types::{
        literal_type_definition::LiteralTypeDefinition,
        type_match::{TypeSatisfiesValueContainer, TypeSuperset},
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

impl TypeSuperset<LiteralTypeDefinition> for LiteralTypeDefinition {
    fn is_superset_of(&self, other: &LiteralTypeDefinition) -> bool {
        self == other
    }
}

impl TypeSatisfiesValueContainer for LiteralTypeDefinition {
    fn satisfies_value_container(&self, value: &ValueContainer) -> bool {
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
            type_match::TypeSatisfiesValueContainer,
        },
        values::core_values::integer::Integer,
    };

    #[test]
    fn integer() {
        let integer = LiteralTypeDefinition::Integer(Integer::new(42));
        assert!(integer.satisfies_value_container(&Integer::new(42).into()));

        let integer_u8 = LiteralTypeDefinition::TypedInteger(42u8.into());
        assert!(integer_u8.satisfies_value_container(&42u8.into()));

        let integer_wrong = LiteralTypeDefinition::Integer(Integer::new(43));
        assert!(
            !integer_wrong.satisfies_value_container(&Integer::new(42).into())
        );
    }
}
