use crate::{
    datex_proxy::{
        DatexProxyTypes, DatexValueContainerProxy,
        DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError,
    },
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::memory::Memory,
    types::{r#type::Type, type_definition::TypeDefinition},
    values::value_container::ValueContainer,
};

impl DatexValueContainerProxyInfallibleSerialize for ValueContainer {
    fn to_value_container(self) -> ValueContainer {
        self
    }
}
impl DatexValueContainerProxySerialize for ValueContainer {
    fn try_to_value_container(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self)
    }
}
impl DatexValueContainerProxyDeserialize for ValueContainer {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        Ok(value)
    }
}

impl DatexProxyTypes for ValueContainer {
    fn datex_type(_memory: &mut Memory) -> Type {
        Type::Alias(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Unknown.into()).into(),
        )
    }
}

/// Custom [DatexValueContainerProxy] for [ValueContainer] - just return as is
impl DatexValueContainerProxy for ValueContainer {}
