//! This module contains the implementation of the [DatexValueProxy] trait for the Option type.
//! As `Option<T>` is a special Rust type that has no direct equivalent in DATEX, it is represented as a union of `null` and `T` in DATEX.
//! As `Some(None)` would be indistinguishable from `None` when serialized (both would be represented as `null`), we use a impl marker to
//! represent `Some(None)` in DATEX.
use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    shared_values::errors::KeyNotFoundError,
    types::{
        r#type::Type,
        type_definition::{
            collection::{
                CollectionTypeDefinition,
                type_definition::list::ListCollectionTypeDefinition,
            },
            impl_type::ImplTypeDefinition,
        },
    },
    values::{
        cast::special_rust_types::{rust_none_marker, rust_some_marker},
        core_value::CoreValue,
        core_values::list::List,
        value::Value,
        value_container::ValueContainer,
    },
};

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::{TypeDefinition, union::UnionTypeDefinition},
};

impl Value {
    pub fn rust_none() -> Self {
        Self::new(
            CoreValue::Null,
            Some(TypeDefinition::ImplType(ImplTypeDefinition::new(
                Type::NULL,
                vec![rust_none_marker()],
            ))),
        )
    }

    pub fn is_rust_none(&self) -> bool {
        match &self.custom_type {
            Some(TypeDefinition::ImplType(impl_type)) => impl_type
                .impl_markers
                .iter()
                .any(|marker| marker == &rust_none_marker()),
            _ => false,
        }
    }

    pub fn rust_some(value: Value) -> Self {
        Self::new(
            CoreValue::List(List::from(vec![ValueContainer::from(value)])),
            Some(TypeDefinition::ImplType(ImplTypeDefinition::new(
                Type::core(CoreLibBaseTypeId::Any),
                vec![rust_some_marker()],
            ))),
        )
    }

    pub fn is_rust_some(&self) -> bool {
        match &self.custom_type {
            Some(TypeDefinition::ImplType(impl_type)) => impl_type
                .impl_markers
                .iter()
                .any(|marker| marker == &rust_some_marker()),
            _ => false,
        }
    }

    pub fn into_rust_some_inner(self) -> Result<Value, TryFromDatexValueError> {
        if !self.is_rust_some() {
            return Err(TryFromDatexValueError(
                "Expected Rust Some".to_string(),
            ));
        }
        let CoreValue::List(list) = self.inner else {
            return Err(TryFromDatexValueError(
                "Expected Rust Some inner list".to_string(),
            ));
        };

        let mut values = list.into_iter();
        let value = values.next().ok_or_else(|| {
            TryFromDatexValueError("Expected Rust Some inner value".to_string())
        })?;

        if values.next().is_some() {
            return Err(TryFromDatexValueError(
                "Expected exactly one Rust Some inner value".to_string(),
            ));
        }
        match value {
            ValueContainer::Local(value) => Ok(value),
            _ => Err(TryFromDatexValueError(
                "Expected local Rust Some inner value".to_string(),
            )),
        }
    }
}

impl<T: DatexValueProxy> DatexValueProxy for Option<T> {}

impl<T: DatexValueProxyInfallibleSerialize> DatexValueProxyInfallibleSerialize
    for Option<T>
{
    fn to_value(self) -> Value {
        match self {
            None => Value::rust_none(),
            Some(v) => {
                let value = v.to_value();

                if value.is_rust_none() || value.is_rust_some() {
                    Value::rust_some(value)
                } else {
                    value
                }
            }
        }
    }
}

impl<T: DatexValueProxy> DatexValueProxySerialize for Option<T> {
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        match self {
            None => Ok(Value::rust_none()),
            Some(v) => {
                let value = v.try_to_value()?;

                if value.is_rust_none() || value.is_rust_some() {
                    Ok(Value::rust_some(value))
                } else {
                    Ok(value)
                }
            }
        }
    }
}

impl<T: DatexValueProxy> DatexValueProxyDeserialize for Option<T> {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        if value.is_rust_none() {
            return Ok(None);
        }

        if value.is_rust_some() {
            return Ok(Some(T::try_from_value(value.into_rust_some_inner()?)?));
        }

        Ok(Some(T::try_from_value(value)?))
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

impl<T> DatexProxyTypes for Option<T>
where
    T: DatexProxyTypes,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        let inner_type = T::datex_type(memory);
        let rust_none_type = Type::Definition(
            TypeDefinition::ImplType(ImplTypeDefinition::new(
                Type::NULL,
                vec![rust_none_marker()],
            ))
            .into(),
        );

        let rust_some_type = Type::Definition(
            TypeDefinition::ImplType(ImplTypeDefinition::new(
                Type::Definition(
                    TypeDefinition::Collection(CollectionTypeDefinition::List(
                        ListCollectionTypeDefinition(Box::new(
                            inner_type.clone(),
                        )),
                    ))
                    .into(),
                ),
                vec![rust_some_marker()],
            ))
            .into(),
        );

        Type::Definition(
            TypeDefinition::Union(UnionTypeDefinition(vec![
                rust_none_type,
                inner_type,
                rust_some_type,
            ]))
            .into(),
        )
    }
}
