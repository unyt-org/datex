use crate::{
    datex_proxy::{
        DatexValueProxy, DatexValueProxyDeserialize,
        DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
        TryFromDatexValueError, TryToDatexValueError,
    },
    values::value::Value,
};
use crate::datex_proxy::DatexProxyTypes;
use crate::libs::core::type_id::CoreLibBaseTypeId;
use crate::runtime::memory::Memory;
use crate::shared_values::{ReferenceMutability, SharedContainer, SharedContainerMutability, SharedContainerOwnership};
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;
use crate::types::type_definition_with_metadata::{TypeDefinitionWithMetadata, TypeMetadata};

impl DatexValueProxyInfallibleSerialize for Value {
    fn to_value(self) -> Value {
        self
    }
}
impl DatexValueProxySerialize for Value {
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        Ok(self)
    }
}
impl DatexValueProxyDeserialize for Value {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        Ok(value)
    }
}

impl DatexProxyTypes for Value {
    fn datex_type(memory: &mut Memory) -> Type {
        Type::Alias(TypeDefinition::Core(CoreLibBaseTypeId::Unknown.into()).into())
    }
}

impl DatexValueProxy for Value {}
