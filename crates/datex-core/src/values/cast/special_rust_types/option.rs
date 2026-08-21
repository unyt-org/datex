//! Implements [DatexValueProxy] for [Option<T>] where T: [DatexValueProxy].
//! As `Option<T>` is a special Rust type that has no direct equivalent in DATEX, it is represented as a union of `null` and `T` in DATEX.
//! As `Some(None)` would be indistinguishable from `None` when serialized (both would be represented as `null`), we use a tagged type representation for
//! `Option<T>` in DATEX, where `None` is represented as a tagged type with the tag "None(null)", and `Some(T)` is represented as a tagged type with the tag "Some" and
//! an inner type of `T`.

use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::KeyNotFoundError,
    types::{
        r#type::Type,
        type_definition::{TypeDefinition, union::UnionTypeDefinition},
    },
    values::value::Value,
};

impl<T: DatexValueProxy> DatexValueProxy for Option<T> {}
impl<T: DatexValueProxyInfallibleSerialize> DatexValueProxyInfallibleSerialize
    for Option<T>
{
    fn to_value(self, context: &mut SharedReferencesCache) -> Value {
        Value::boxed(match self {
            None => Value::null(),
            Some(value) => Box::new(value).to_value(context),
        })
    }
}

impl<T: DatexValueProxy> DatexValueProxySerialize for Option<T> {
    fn try_to_value(
        self,
        context: &mut SharedReferencesCache,
    ) -> Result<Value, TryToDatexValueError> {
        match self {
            None => Ok(Value::boxed(Value::null())),
            Some(value) => {
                Ok(Value::boxed(Box::new(value).try_to_value(context)?))
            }
        }
    }
}

impl<T: DatexValueProxyDeserialize> DatexValueProxyDeserialize for Option<T> {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        // directly check for unboxed null value, DATEX interprets this as None
        if value.is_null() {
            return Ok(None);
        }

        match value.unbox() {
            // if the value is a boxed container, it came out of Rust serialization
            Ok(container) => {
                // matching the serialize
                if container.is_null() {
                    Ok(None)
                } else {
                    Ok(Some(T::try_from_value_container(container)?))
                }
            }
            Err(value) => Ok(Some(T::try_from_value(value)?)),
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
impl<T> DatexProxyTypes for Option<T>
where
    T: DatexProxyTypes,
{
    /// Returns the container type definition for `Option<T>`, which is a union of `null` and the type definition of `T`,
    /// wrapped in a container
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
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
    use crate::values::{core_values::integer::Integer, value::Value};

    #[test]
    fn to_value() {
        let some_option: Option<Integer> = Some(Integer::new(1));
        let none_option: Option<Integer> = None;

        let some_value: Value = some_option.to_value_without_context();
        let none_value: Value = none_option.to_value_without_context();

        assert_eq!(
            some_value,
            Value::boxed(Integer::new(1).to_value_without_context())
        );
        assert_eq!(none_value, Value::boxed(Value::null()));
    }

    #[test]
    fn from_value() {
        let some_value: Value =
            Value::boxed(Integer::new(1).to_value_without_context());
        let some_option: Option<Integer> =
            Option::try_from_value(some_value).unwrap();
        assert_eq!(some_option, Some(Integer::new(1)));

        let none_value: Value = Value::boxed(Value::null());
        let none_option: Option<Integer> =
            Option::try_from_value(none_value).unwrap();
        assert_eq!(none_option, None);
    }
    #[test]
    fn datex_type() {
        let option_type = Option::<Integer>::datex_type_without_context();
        option_type.with_collapsed_type_definition(|td| {
            assert!(matches!(td, TypeDefinition::Box(_)));
        });
    }
}
