use crate::{
    datex_proxy::ToDatexNativeValueContainer,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::{value::Value, value_container::ValueContainer},
};
use core::{
    any::Any,
    fmt::{Debug, Formatter},
    ops::Deref,
};
mod datex_native_trait;
mod serde_dif;
#[cfg(feature = "decompiler")]
mod to_datex_expression_data;
mod value_access;

use crate::{
    libs::core::type_id::CoreLibTypeId, traits::try_clone::TryClone,
    values::core_value::CoreValue,
};
pub use datex_native_trait::*;

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

impl TryClone for NativeCoreValue {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        self.value.deref().try_clone()
    }
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

    /// Attempt to downcast the native value to a specific type.
    /// Returns `Some(&T)` if the downcast is successful, or `None` if it fails.
    pub fn try_as<T: 'static>(&self) -> Option<&T> {
        self.value.as_any().downcast_ref::<T>()
    }

    /// Attempt to downcast the native value to a specific type.
    /// Returns `Some(&mut T)` if the downcast is successful, or `None` if it fails.
    pub fn try_into_value<T: 'static>(self) -> Option<T> {
        match self.into_any().downcast::<T>() {
            Ok(boxed) => Some(*boxed),
            Err(_original) => None,
        }
    }
}

impl Clone for NativeCoreValue {
    fn clone(&self) -> Self {
        match self.try_clone().unwrap() {
            CoreValue::Native(n) => n,
            _ => unreachable!(),
        }
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
