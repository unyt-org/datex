use crate::{
    datex_proxy::{
        DatexProxyTypes, DatexValueProxy, DatexValueProxyDeserialize,
        DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
        TryFromDatexValueError, TryToDatexValueError,
    },
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::memory::Memory,
    types::{r#type::Type, type_definition::TypeDefinition},
    values::{core_value::CoreValue, value::Value},
};

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
        Type::Alias(
            TypeDefinition::Core(CoreLibBaseTypeId::Unknown.into()).into(),
        )
    }
}

impl DatexValueProxy for CoreValue {}
