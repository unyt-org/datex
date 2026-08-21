use core::{
    any::Any,
    fmt::{Debug, Formatter},
};

use crate::{
    datex_proxy::DatexValueProxySerialize,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::value::Value,
};

pub trait DatexNative: Any + DatexValueProxySerialize {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn to_native_value(
        self: Box<Self>,
        cache: &mut SharedReferencesCache,
    ) -> Value;
}

pub struct NativeCoreValue {
    pub value: Box<dyn DatexNative + 'static>,
}

impl NativeCoreValue {
    pub fn new<T>(value: T) -> Self
    where
        T: DatexNative + 'static,
    {
        NativeCoreValue {
            value: Box::new(value),
        }
    }

    pub fn as_any(&self) -> &dyn Any {
        self.value.as_ref().as_any()
    }
    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        self.value.as_mut().as_any_mut()
    }

    pub fn to_native_value(self, cache: &mut SharedReferencesCache) -> Value {
        self.value.to_native_value(cache)
    }
}

impl Clone for NativeCoreValue {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Debug for NativeCoreValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "[[ native value ]]")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        libs::core::type_id::CoreLibBaseTypeId,
        runtime::cache::shared_references_cache::SharedReferencesCache,
        types::type_definition::TypeDefinition,
        values::{core_value::CoreValue, core_values::native::NativeCoreValue},
    };

    #[test]
    fn serde() {
        let val = NativeCoreValue::new(String::from("xx"));
        let ser = val.to_native_value(&mut SharedReferencesCache::default());
        assert_eq!(
            ser.custom_type().expect("custom type should be present"),
            &TypeDefinition::core(CoreLibBaseTypeId::Text),
        );
        assert_eq!(
            ser.inner,
            CoreValue::Native(NativeCoreValue::new("xx".to_owned()))
        );
    }
}
