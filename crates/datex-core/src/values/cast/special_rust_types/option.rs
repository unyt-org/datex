//! Implements [DatexValueProxy] for [Option<T>] where T: [DatexValueProxy].
//! As `Option<T>` is a special Rust type that has no direct equivalent in DATEX, it is represented as a union of `null` and `T` in DATEX.
//! As `Some(None)` would be indistinguishable from `None` when serialized (both would be represented as `null`), we use a tagged type representation for
//! `Option<T>` in DATEX, where `None` is represented as a tagged type with the tag "None(null)", and `Some(T)` is represented as a tagged type with the tag "Some" and
//! an inner type of `T`.

use core::any::Any;
use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    prelude::*,
    shared_values::errors::KeyNotFoundError,
    types,
    types::r#type::Type,
    values::{core_value::CoreValue, value::Value},
};

use crate::types::type_definition::{
    TypeDefinition, union::UnionTypeDefinition,
};

impl<T: DatexValueProxy<C>, C> DatexValueProxy<C> for Option<T> {}
impl<T: DatexValueProxyInfallibleSerialize<C>, C>
    DatexValueProxyInfallibleSerialize<C> for Option<T>
{
    fn to_value(self, context: &mut C) -> Value {
        Value::boxed(match self {
            None => Value::null(),
            Some(value) => value.to_value(context),
        })
    }
}

impl<T: DatexValueProxy<C>, C> DatexValueProxySerialize<C> for Option<T> {
    fn try_to_value(
        self,
        context: &mut C,
    ) -> Result<Value, TryToDatexValueError> {
        match self {
            None => Ok(Value::boxed(Value::null())),
            Some(value) => Ok(Value::boxed(value.try_to_value(context)?)),
        }
    }
}

impl<T: DatexValueProxyDeserialize> DatexValueProxyDeserialize for Option<T> {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        if matches!(value.inner, CoreValue::Null) {
            Ok(None)
        } else {
            T::try_from_value(value).map(Some)
        }
    }

    fn try_from_map_property(
        value: Result<Value, KeyNotFoundError>,
    ) -> Result<Self, TryFromDatexValueError> {
        match value {
            Ok(value) => Self::try_from_value(value),
            Err(_) => Ok(None),
        }
    }
}

/// TODO: only wrap nested Option<Option<T>> into container. Single option can be mapped directly to X|null
impl<T, C> DatexProxyTypes<C> for Option<T>
where
    T: DatexProxyTypes<C>,
{
    /// Returns the container type definition for `Option<T>`, which is a union of `null` and the type definition of `T`,
    /// wrapped in a container
    fn datex_type(memory: &mut C) -> Type {
        let inner_type = T::datex_type(memory);
        Type::Definition(
            TypeDefinition::Box(Box::new(
                TypeDefinition::Union(UnionTypeDefinition(vec![
                    Type::NULL,
                    inner_type,
                ]))
                .into(),
            ))
            .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        datex_proxy::DatexValueProxyInfallibleSerialize,
        values::{core_values::integer::Integer, value::Value},
    };

    #[test]
    fn to_value() {
        let some_option: Option<Integer> = Some(Integer::new(1));
        let none_option: Option<Integer> = None;

        let some_value: Value = some_option.to_value_without_context();
        let none_value: Value = none_option.to_value_without_context();
    }

    #[test]
    fn from_value() {
        let some_value: Value =
            Value::boxed(Integer::new(1).to_value_without_context());
        let some_option: Option<Integer> =
            Option::try_from_value(some_value).unwrap();
        assert_eq!(some_option, Some(Integer::new(1)));
    }
    #[test]
    fn datex_type() {
        let option_type = Option::<Integer>::datex_type_without_context();
        option_type.with_collapsed_type_definition(|td| {
            assert!(matches!(td, TypeDefinition::Box(_)));
        });
    }
}
