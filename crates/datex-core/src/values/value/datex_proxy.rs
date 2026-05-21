use crate::{
    datex_proxy::{
        DatexValueProxy, DatexValueProxyDeserialize,
        DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
        TryFromDatexValueError, TryToDatexValueError,
    },
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

impl DatexValueProxy for Value {}
