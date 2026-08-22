use crate::{
    datex_proxy::{
        DatexProxyType, DatexValueContainerProxy,
        DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError,
    },
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::{r#type::Type, type_definition::TypeDefinition},
    values::value_container::ValueContainer,
};

use crate::prelude::*;
impl DatexValueContainerProxyInfallibleSerialize for ValueContainer {
    fn boxed_to_value_container(
        self: Box<Self>,
        _context: &mut SharedReferencesCache,
    ) -> ValueContainer {
        *self
    }
}
impl DatexValueContainerProxySerialize for ValueContainer {
    fn try_boxed_to_value_container(
        self: Box<Self>,
        _context: &mut SharedReferencesCache,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(*self)
    }
}
impl DatexValueContainerProxyDeserialize for ValueContainer {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        Ok(value)
    }
}

impl DatexProxyType for ValueContainer {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Any.into()).into(),
        )
    }
}

/// Custom [DatexValueContainerProxy] for [ValueContainer] - just return as is
impl DatexValueContainerProxy for ValueContainer {}
