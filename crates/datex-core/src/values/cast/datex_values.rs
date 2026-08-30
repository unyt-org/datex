//! Implements [DatexValueProxy] for [CoreValue](crate::values::core_values) implementation. That allows to convert e.g. [Endpoint] to [Value] and back.
//! Also implements [GetDatexType] to provide the correct [Type] for each implementation.

use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    types::{
        entities::entity_type_definition::EntityTypeDefinition, r#type::Type,
    },
    values::{
        core_values::{
            boolean::Boolean, callable::Callable, decimal::Decimal,
            endpoint::Endpoint, integer::Integer, list::List, map::Map,
            native::DatexNative, range::Range, text::Text,
        },
        value::Value,
    },
};
use core::any::Any;

use crate::{
    libs::core::type_id::CoreLibTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::TypeDefinition,
    values::borrowed_value_container::{
        AsBorrowed, AsBorrowedMut, BorrowedValueContainer,
        BorrowedValueContainerMut,
    },
};
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::traits::get_datex_type::GetDatexType;

/// Implements [DatexValueProxy] for a [CoreValue](crate::values::core_values) implementation.
/// This allows to convert e.g. [Endpoint] to [ValueContainer] and back.
/// Also implements [GetDatexType] to provide the correct [Type] for each implementation.
/// The `gen` param defines, for which concrete context to impl the serialization traits.
macro_rules! impl_datex_direct_via_value_container {
    ($type:ty, $dx_type:expr) => {
        impl GetCoreLibTypeId for $type {
            fn core_lib_type_id(&self) -> CoreLibTypeId {
                $dx_type.into()
            }
        }
        
        impl GetDatexType for $type {
            fn datex_type(_context: &mut SharedReferencesCache) -> Type {
                Type::Definition(TypeDefinition::CoreType($dx_type.into()).into())
            }
        }

        impl<'a> AsBorrowed<'a> for $type {
            fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
                BorrowedValueContainer::native_borrowed_only_structural(self)
            }
        }
        impl<'a> AsBorrowedMut<'a> for $type {
            fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a> {
                BorrowedValueContainerMut::native_borrowed_only_structural(self)
            }
        }
    };
}

impl_datex_direct_via_value_container!(Endpoint, CoreLibBaseTypeId::Endpoint);
impl_datex_direct_via_value_container!(Map, CoreLibBaseTypeId::Map);
impl_datex_direct_via_value_container!(List, CoreLibBaseTypeId::List);
impl_datex_direct_via_value_container!(Range, CoreLibBaseTypeId::Range);
impl_datex_direct_via_value_container!(Type, CoreLibBaseTypeId::Type);
impl_datex_direct_via_value_container!(
    EntityTypeDefinition,
    CoreLibBaseTypeId::Any
);
impl_datex_direct_via_value_container!(Callable, CoreLibBaseTypeId::Callable);
impl_datex_direct_via_value_container!(Integer, CoreLibBaseTypeId::Integer);
impl_datex_direct_via_value_container!(Decimal, CoreLibBaseTypeId::Decimal);
impl_datex_direct_via_value_container!(Text, CoreLibBaseTypeId::Text);
impl_datex_direct_via_value_container!(Boolean, CoreLibBaseTypeId::Boolean);
// impl_datex_direct_via_value_container!(Instant, CoreLibBaseTypeId::Instant);

#[cfg(test)]
mod tests {
    use crate::{
        values::{
            core_value::CoreValue, core_values::endpoint::Endpoint,
            value::Value,
        },
    };

    #[test]
    fn to_value() {
        let endpoint = Endpoint::new("@jonas");
        let value = Value::native_only_structural(endpoint.clone());
        assert!(matches!(
            value.inner,
            CoreValue::Endpoint(ref e) if e == &endpoint
        ));
    }

    #[test]
    fn try_boxed_to_value() {
        let endpoint = Endpoint::new("@jonas");
        let value = Value::native_only_structural(endpoint.clone());
        assert!(matches!(
            value.inner,
            CoreValue::Endpoint(ref e) if e == &endpoint
        ));
    }

    #[test]
    fn try_from_value() {
        let endpoint = Endpoint::new("@jonas");
        let value = Value::native_only_structural(endpoint.clone());
        let result: Endpoint = value.try_into().unwrap();
        assert_eq!(result, endpoint);
    }
}
