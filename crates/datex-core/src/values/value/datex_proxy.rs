use crate::{
    datex_proxy::{
        DatexProxyType, DatexValueProxy, DatexValueProxyDeserialize,
        DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
        TryFromDatexValueError, TryToDatexValueError,
    },
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::{r#type::Type, type_definition::TypeDefinition},
    values::value::Value,
};

impl DatexValueProxyInfallibleSerialize for Value {
    fn boxed_to_value(
        self: Box<Self>,
        _context: &mut SharedReferencesCache,
    ) -> Value {
        *self
    }
}
impl DatexValueProxySerialize for Value {
    fn try_boxed_to_value(
        self: Box<Self>,
        _context: &mut SharedReferencesCache,
    ) -> Result<Value, TryToDatexValueError> {
        Ok(*self)
    }
}
impl DatexValueProxyDeserialize for Value {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        Ok(value)
    }
}

impl DatexProxyType for Value {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Any.into()).into(),
        )
    }
}

impl DatexValueProxy for Value {}
