use crate::{
    datex_proxy::{
        DatexProxyTypes, DatexValueProxy, DatexValueProxyDeserialize,
        DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
        TryFromDatexValueError, TryToDatexValueError,
    },
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::memory::Memory,
    types::{r#type::Type, type_definition::TypeDefinition},
    values::value::Value,
};

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
    fn datex_type(_memory: &mut Memory) -> Type {
        Type::Alias(
            TypeDefinition::Core(CoreLibBaseTypeId::Unknown.into()).into(),
        )
    }
}

impl DatexValueProxy for Value {}
