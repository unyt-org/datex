//! Implements [TryFrom] for DATEX [CoreValue] and [Value] types. This allows to convert e.g. [CoreValue::Integer] to [Integer].
use crate::{
    prelude::*,
    types::{
        entities::entity_type_definition::EntityTypeDefinition, r#type::Type,
    },
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean,
            callable::Callable,
            decimal::{Decimal, typed_decimal::TypedDecimal},
            endpoint::Endpoint,
            integer::{Integer, typed_integer::TypedInteger},
            list::List,
            map::Map,
            range::Range,
            text::Text,
        },
        value::{
            Value,
            borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut},
        },
    },
};
use crate::traits::convert_core_value::ConvertCoreValue;

/// Implements [TryFrom] for each [CoreValue] variant to its corresponding type. This allows to convert e.g. [CoreValue::Integer] to [Integer].
macro_rules! impl_try_from_core_value {
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl ConvertCoreValue for $type {
                fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue> {
                    match value {
                        CoreValue::$variant(v) => Ok(v),
                        _ => Err(value),
                    }
                }

                fn try_borrow_from_core_value(value: &CoreValue) -> Result<&$type, ()> {
                    match value {
                        CoreValue::$variant(v) => Ok(v),
                        _ => Err(()),
                    }
                }

                fn try_borrow_mut_from_core_value(value: &mut CoreValue) -> Result<&mut $type, ()> {
                    match value {
                        CoreValue::$variant(v) => Ok(v),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, $type> {
                type Error = ();
                fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
                    match value {
                        BorrowedCoreValue::$variant(v) => Ok(v),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, $type> {
                type Error = ();
                fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
                    match value {
                        BorrowedCoreValueMut::$variant(v) => Ok(v),
                        _ => Err(()),
                    }
                }
            }

        )*
    };
}

impl_try_from_core_value! {
    Integer             => Integer,
    TypedInteger        => TypedInteger,
    Decimal             => Decimal,
    TypedDecimal        => TypedDecimal,
    Boolean             => Boolean,
    Endpoint            => Endpoint,
    Text                => Text,
    List                => List,
    Map                 => Map,
    Type                => Type,
    EntityTypeDefinition => EntityTypeDefinition,
    Range               => Range,
    Callable            => Callable,
}

#[cfg(test)]
mod tests {
    use core::assert_matches;

    use crate::{
        values::{
            core_value::CoreValue,
            core_values::{integer::Integer, text::Text},
            value::Value,
        },
    };

    #[test]
    fn try_from_core_value() {
        let int_value = CoreValue::Integer(Integer::new(42));
        let int: Integer = int_value.try_into_value().unwrap();
        assert_eq!(int, Integer::new(42));

        let text_value = CoreValue::Text(Text::new("Hello, DATEX!"));
        let text: Text = text_value.try_into_value().unwrap();
        assert_eq!(text, Text::new("Hello, DATEX!"));
    }

    #[test]
    fn try_from_core_value_wrong_type() {
        let int_value = CoreValue::Integer(Integer::new(42));
        let result = int_value.try_into_value::<Text>();
        assert_matches!(result, Err(_));
    }

    #[test]
    fn try_from_core_value_ref() {
        let int_value = CoreValue::Integer(Integer::new(42));
        let int_ref = int_value.try_as::<Integer>().unwrap();
        assert_eq!(*int_ref, Integer::new(42));

        let text_value = CoreValue::Text(Text::new("Hello, DATEX!"));
        let text_ref: &Text = (&text_value).try_as().unwrap();
        assert_eq!(*text_ref, Text::new("Hello, DATEX!"));
    }

    #[test]
    fn try_from_core_value_mut_ref() {
        let mut int_value = CoreValue::Integer(Integer::new(42));
        let int_mut_ref = int_value.try_as_mut::<Integer>().unwrap();
        *int_mut_ref = Integer::new(100);
        assert_eq!(*int_mut_ref, Integer::new(100));
        assert_eq!(int_value, CoreValue::Integer(Integer::new(100)));
    }

    #[test]
    fn try_from_value() {
        let value = Value::from(CoreValue::Integer(Integer::new(42)));
        let int: Integer = value.try_into_value().unwrap();
        assert_eq!(int, Integer::new(42));
    }
}
