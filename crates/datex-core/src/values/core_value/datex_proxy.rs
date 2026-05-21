use crate::{
    datex_proxy::{
        DatexValueProxy, DatexValueProxyDeserialize,
        DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
        TryFromDatexValueError, TryToDatexValueError,
    },
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

impl DatexValueProxy for CoreValue {}
