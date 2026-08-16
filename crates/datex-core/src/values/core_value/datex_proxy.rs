use crate::{
    datex_proxy::{
        DatexProxyTypes, DatexValueProxy, DatexValueProxyDeserialize,
        DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
        TryFromDatexValueError, TryToDatexValueError,
    },
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::{r#type::Type, type_definition::TypeDefinition},
    values::{core_value::CoreValue, value::Value},
};

impl DatexValueProxyInfallibleSerialize<()> for CoreValue {
    fn to_value(self, _context: &mut ()) -> Value {
        Value::from(self)
    }
}
impl DatexValueProxySerialize<()> for CoreValue {
    fn try_to_value(
        self,
        _context: &mut (),
    ) -> Result<Value, TryToDatexValueError> {
        Ok(self.to_value(_context))
    }
}
impl DatexValueProxyDeserialize for CoreValue {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        Ok(value.inner)
    }
}

impl DatexProxyTypes<()> for CoreValue {
    fn datex_type(_context: &mut ()) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Any.into()).into(),
        )
    }
}

impl DatexValueProxy<()> for CoreValue {}
