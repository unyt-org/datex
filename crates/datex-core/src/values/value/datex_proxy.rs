use core::any::Any;
use crate::{
    datex_proxy::{
        DatexProxyTypes, DatexValueProxy, DatexValueProxyDeserialize,
        DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
        TryFromDatexValueError, TryToDatexValueError,
    },
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::{r#type::Type, type_definition::TypeDefinition},
    values::value::Value,
};
use crate::datex_proxy::derive_datex_proxy_types_default;

impl DatexValueProxyInfallibleSerialize<()> for Value {
    fn to_value(self, _context: &mut ()) -> Value {
        self
    }
}
impl DatexValueProxySerialize<()> for Value {
    fn try_to_value(
        self,
        _context: &mut (),
    ) -> Result<Value, TryToDatexValueError> {
        Ok(self)
    }
}
impl DatexValueProxyDeserialize for Value {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        Ok(value)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl DatexProxyTypes<()> for Value {
    fn datex_type(_context: &mut ()) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Any.into()).into(),
        )
    }
}
derive_datex_proxy_types_default!(Value);

impl DatexValueProxy<()> for Value {}
