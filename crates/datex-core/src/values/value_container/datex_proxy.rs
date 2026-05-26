use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError,
    },
    values::value_container::ValueContainer,
};
use crate::datex_proxy::DatexProxyTypes;
use crate::libs::core::type_id::CoreLibBaseTypeId;
use crate::runtime::memory::Memory;
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;

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
    fn datex_type(memory: &mut Memory) -> Type {
        Type::Alias(TypeDefinition::Core(CoreLibBaseTypeId::Unknown.into()).into())
    }
}


/// Custom [DatexValueContainerProxy] for [ValueContainer] - just return as is
impl DatexValueContainerProxy for ValueContainer {}
