use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    prelude::*,
    types::r#type::Type,
    values::value::Value,
};

use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl<T> DatexValueProxy for Box<T> where T: DatexValueProxy {}

impl<T> DatexValueProxySerialize for Box<T>
where
    T: DatexValueProxySerialize,
{
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        (*self).try_to_value()
    }
}

impl<T> DatexValueProxyInfallibleSerialize for Box<T>
where
    T: DatexValueProxyInfallibleSerialize,
{
    fn to_value(self) -> Value {
        (*self).to_value()
    }
}

impl<T> DatexValueProxyDeserialize for Box<T>
where
    T: DatexValueProxyDeserialize,
{
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        Ok(Box::new(T::try_from_value(value)?))
    }
}

impl<T> DatexProxyTypes for Box<T>
where
    T: DatexProxyTypes,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        T::datex_type(memory)
    }
}
