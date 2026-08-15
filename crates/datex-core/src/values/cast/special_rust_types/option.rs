//! This module contains the implementation of the [DatexValueProxy] trait for the Option type.
//! As `Option<T>` is a special Rust type that has no direct equivalent in DATEX, it is represented as a union of `null` and `T` in DATEX.
//! As `Some(None)` would be indistinguishable from `None` when serialized (both would be represented as `null`), we use a tagged type representation for
//! `Option<T>` in DATEX, where `None` is represented as a tagged type with the tag "None(null)", and `Some(T)` is represented as a tagged type with the tag "Some" and
//! an inner type of `T`.
use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    prelude::*,
    shared_values::errors::KeyNotFoundError,
    types::{
        r#type::Type,
        type_definition::{
            list::ListTypeDefinition, tagged_type::TaggedTypeDefinition,
        },
    },
    values::{
        core_value::CoreValue, core_values::list::List, value::Value,
        value_container::ValueContainer,
    },
};

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::{TypeDefinition, union::UnionTypeDefinition},
};
// impl Type {
//     pub fn rust_none() -> Self {
//         Type::Definition(
//             TypeDefinition::ImplType(ImplTypeDefinition::new(
//                 Type::NULL,
//                 vec![rust_none_marker()],
//             ))
//             .into(),
//         )
//     }
//     pub fn rust_some(inner_type: Type) -> Self {
//         Type::Definition(
//             TypeDefinition::ImplType(ImplTypeDefinition::new(
//                 Type::Definition(
//                     TypeDefinition::Collection(CollectionTypeDefinition::List(
//                         ListCollectionTypeDefinition(Box::new(inner_type)),
//                     ))
//                     .into(),
//                 ),
//                 vec![rust_some_marker()],
//             ))
//             .into(),
//         )
//     }
//     pub fn is_rust_none(&self) -> bool {
//         self.with_collapsed_type_definition(|def| match def {
//             TypeDefinition::ImplType(impl_type) => impl_type
//                 .impl_markers
//                 .iter()
//                 .any(|marker| marker == &rust_none_marker()),
//             _ => false,
//         })
//     }
// }
// impl Value {
//     pub fn rust_none() -> Self {
//         Self::new(
//             CoreValue::Null,
//             Some(TypeDefinition::ImplType(ImplTypeDefinition::new(
//                 Type::NULL,
//                 vec![rust_none_marker()],
//             ))),
//         )
//     }

//     pub fn is_rust_none(&self) -> bool {
//         match &self.custom_type {
//             Some(TypeDefinition::ImplType(impl_type)) => impl_type
//                 .impl_markers
//                 .iter()
//                 .any(|marker| marker == &rust_none_marker()),
//             _ => false,
//         }
//     }

//     pub fn rust_some(value: Value) -> Self {
//         Self::new(
//             CoreValue::List(List::from(vec![ValueContainer::from(value)])),
//             Some(TypeDefinition::ImplType(ImplTypeDefinition::new(
//                 Type::core(CoreLibBaseTypeId::Any),
//                 vec![rust_some_marker()],
//             ))),
//         )
//     }

//     pub fn is_rust_some(&self) -> bool {
//         match &self.custom_type {
//             Some(TypeDefinition::ImplType(impl_type)) => impl_type
//                 .impl_markers
//                 .iter()
//                 .any(|marker| marker == &rust_some_marker()),
//             _ => false,
//         }
//     }

//     pub fn into_rust_some_inner(self) -> Result<Value, TryFromDatexValueError> {
//         if !self.is_rust_some() {
//             return Err(TryFromDatexValueError(
//                 "Expected Rust Some".to_string(),
//             ));
//         }
//         let CoreValue::List(list) = self.inner else {
//             return Err(TryFromDatexValueError(
//                 "Expected Rust Some inner list".to_string(),
//             ));
//         };

//         let mut values = list.into_iter();
//         let value = values.next().ok_or_else(|| {
//             TryFromDatexValueError("Expected Rust Some inner value".to_string())
//         })?;

//         if values.next().is_some() {
//             return Err(TryFromDatexValueError(
//                 "Expected exactly one Rust Some inner value".to_string(),
//             ));
//         }
//         match value {
//             ValueContainer::Local(value) => Ok(value),
//             _ => Err(TryFromDatexValueError(
//                 "Expected local Rust Some inner value".to_string(),
//             )),
//         }
//     }
// }

// impl<T: DatexValueProxy> DatexValueProxy for Option<T> {}

// impl<T: DatexValueProxyInfallibleSerialize> DatexValueProxyInfallibleSerialize
//     for Option<T>
// {
//     fn to_value(self) -> Value {
//         match self {
//             None => Value::rust_none(),
//             Some(v) => {
//                 let value = v.to_value();

//                 if value.is_rust_none() || value.is_rust_some() {
//                     Value::rust_some(value)
//                 } else {
//                     value
//                 }
//             }
//         }
//     }
// }

// impl<T: DatexValueProxy> DatexValueProxySerialize for Option<T> {
//     fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
//         match self {
//             None => Ok(Value::rust_none()),
//             Some(v) => {
//                 let value = v.try_to_value()?;

//                 if value.is_rust_none() || value.is_rust_some() {
//                     Ok(Value::rust_some(value))
//                 } else {
//                     Ok(value)
//                 }
//             }
//         }
//     }
// }

// impl<T: DatexValueProxy> DatexValueProxyDeserialize for Option<T> {
//     fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
//         if value.is_rust_none() {
//             return Ok(None);
//         }

//         if value.is_rust_some() {
//             return Ok(Some(T::try_from_value(value.into_rust_some_inner()?)?));
//         }

//         Ok(Some(T::try_from_value(value)?))
//     }

//     fn try_from_map_property(
//         value: Result<Value, KeyNotFoundError>,
//     ) -> Result<Self, TryFromDatexValueError> {
//         match value {
//             Ok(value) => Self::try_from_value(value),
//             Err(_) => Ok(None),
//         }
//     }
// }

// impl<T> DatexProxyTypes for Option<T>
// where
//     T: DatexProxyTypes,
// {
//     fn datex_type(memory: &mut SharedReferencesCache) -> Type {
//         let inner_type = T::datex_type(memory);
//         let rust_none_type = Type::rust_none();
//         let rust_some_type = Type::rust_some(inner_type.clone());
//         Type::Definition(
//             TypeDefinition::Union(UnionTypeDefinition(vec![
//                 rust_none_type,
//                 inner_type,
//                 rust_some_type,
//             ]))
//             .into(),
//         )
//     }
// }

impl Type {
    pub fn rust_none() -> Self {
        Type::Definition(
            TypeDefinition::TaggedType(TaggedTypeDefinition {
                tag: "None".to_string(),
                ty: None,
            })
            .into(),
        )
    }

    pub fn rust_some(inner_type: Type) -> Self {
        Type::Definition(
            TypeDefinition::TaggedType(TaggedTypeDefinition {
                tag: "Some".to_string(),
                ty: Some(Box::new(Type::Definition(
                    TypeDefinition::List(ListTypeDefinition(vec![inner_type]))
                        .into(),
                ))),
            })
            .into(),
        )
    }
}

impl Value {
    /// Creates a new Value representing the Rust `None` variant. The custom type of the Value is set to a tagged type with the tag "None" and no inner type.
    pub fn rust_none() -> Self {
        Self::new(
            CoreValue::Null,
            Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
                tag: "None".to_string(),
                ty: None,
            })),
        )
    }

    pub fn is_rust_none(&self) -> bool {
        matches!(
            &self.custom_type,
            Some(TypeDefinition::TaggedType(TaggedTypeDefinition { tag, .. })) if tag == "None" // FIXME maybe worth to add pointer id into tgaged def, or allow impl type def on top
        )
    }

    /// Creates a new Value representing the Rust `Some` variant, wrapping the provided inner value.
    /// The inner value is stored in a list to ensure that it can be distinguished from the `None` variant when serialized, as both `Some(None)` and `None` would otherwise be represented as `null`.
    /// The custom type of the Value is set to a tagged type with the tag "Some" and an inner type of a list containing the type of the provided value.
    pub fn rust_some(value: Value) -> Self {
        Self::new(
            CoreValue::List(List::from(vec![ValueContainer::Local(value)])),
            Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
                tag: "Some".to_string(),
                ty: None,
            })),
        )
    }

    pub fn is_rust_some(&self) -> bool {
        matches!(
            &self.custom_type,
            Some(TypeDefinition::TaggedType(TaggedTypeDefinition { tag, .. })) if tag == "Some" // FIXME see above
        )
    }
}

impl<T: DatexValueProxy> DatexValueProxy for Option<T> {}
impl<T: DatexValueProxyInfallibleSerialize> DatexValueProxyInfallibleSerialize
    for Option<T>
{
    fn to_value(self) -> Value {
        match self {
            None => Value::rust_none(),
            Some(value) => Value::rust_some(value.to_value()),
        }
    }
}

impl<T: DatexValueProxy> DatexValueProxySerialize for Option<T> {
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        match self {
            None => Ok(Value::rust_none()),
            Some(value) => Ok(Value::rust_some(value.try_to_value()?)),
        }
    }
}

impl<T: DatexValueProxy> DatexValueProxyDeserialize for Option<T> {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        if value.is_rust_none() {
            return Ok(None);
        }
        if value.is_rust_some() {
            let list: &List = value.try_as().ok_or_else(|| {
                TryFromDatexValueError(
                    "Invalid value for Some variant: expected a list"
                        .to_string(),
                )
            })?;
            let mut iter = list.iter();
            let value = iter.next().ok_or_else(|| {
                TryFromDatexValueError(
                    "could not find inner value for Some variant".to_string(),
                )
            })?;
            if iter.next().is_some() {
                return Err(TryFromDatexValueError(
                    "Expected exactly one element for Some variant".to_string(),
                ));
            }
            return Ok(Some(T::try_from_value_container(value.clone())?));
        }
        Err(TryFromDatexValueError(
            "Expected None or Some variant".to_string(),
        ))
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
    /// Returns the DATEX type union with the None and Some variants of the Option type.
    /// ty = Some(T) | None(null)
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        let inner_type = T::datex_type(memory);
        Type::Definition(
            TypeDefinition::Union(UnionTypeDefinition(vec![
                Type::rust_none(),
                Type::rust_some(inner_type),
            ]))
            .into(),
        )
    }
}
