use crate::{
    datex_proxy::{
        DatexValueProxy, DatexValueProxyDeserialize,
        DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
        TryFromDatexValueError, TryToDatexValueError,
    },
    values::{core_value::CoreValue, value::Value},
};
use crate::datex_proxy::{DatexProxyTypes, DatexValueContainerProxy};
use crate::datex_proxy::shared::Shared;
use crate::libs::core::type_id::CoreLibBaseTypeId;
use crate::runtime::memory::Memory;
use crate::shared_values::{SharedContainerMutability, SharedContainerOwnership};
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;
use crate::types::type_definition_with_metadata::{TypeDefinitionWithMetadata, TypeMetadata};

impl DatexValueProxyInfallibleSerialize for CoreValue {
    fn to_value(self) -> Value {
        Value::from(self)
    }
}
impl DatexValueProxySerialize for CoreValue {
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        Ok(self.to_value())
    }
}
impl DatexValueProxyDeserialize for CoreValue {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        Ok(value.inner)
    }
}

impl DatexProxyTypes for CoreValue {
    fn datex_type(_memory: &mut Memory) -> Type {
        Type::Alias(TypeDefinition::Core(CoreLibBaseTypeId::Unknown.into()).into())
    }
}


impl DatexValueProxy for CoreValue {}
