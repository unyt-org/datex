use crate::prelude::*;
use core::{
    any::Any,
    fmt::{Debug, Formatter},
};

use crate::{
    datex_proxy::ToDatexNativeValueContainer,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::{value::Value, value_container::ValueContainer},
};
mod datex_native_trait;
#[cfg(feature = "decompiler")]
mod to_datex_expression_data;
mod value_access;

pub use datex_native_trait::*;
use crate::libs::core::type_id::CoreLibTypeId;

impl<T: DatexNative> ToDatexNativeValueContainer for T {
    fn boxed_to_datex_native_value_container(
        self,
        cache: &mut SharedReferencesCache,
    ) -> ValueContainer {
        ValueContainer::Local(Box::new(self).boxed_to_datex_native_value(cache))
    }
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
    pub fn into_any(self) -> Box<dyn Any> {
        self.value
    }

    pub fn to_datex_native_value(
        self,
        cache: &mut SharedReferencesCache,
    ) -> Value {
        self.value.boxed_to_datex_native_value(cache)
    }
    
    pub fn core_lib_type_id(&self) -> CoreLibTypeId {
        self.value.core_lib_type_id()
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

    use crate::prelude::*;
    #[test]
    fn serde() {
        let val = NativeCoreValue::new("xx".to_string());
        let ser =
            val.to_datex_native_value(&mut SharedReferencesCache::default());
        assert_eq!(
            ser.custom_type().expect("custom type should be present"),
            &TypeDefinition::core(CoreLibBaseTypeId::Text),
        );
        assert_eq!(
            ser.inner,
            CoreValue::Native(NativeCoreValue::new("xx".to_string()))
        );
    }
}
