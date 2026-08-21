use std::any::Any;
use crate::datex_proxy::DatexValueProxySerialize;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::values::value::Value;
pub trait DatexNative: Any + DatexValueProxySerialize + DynEq {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convert the boxed native value into a [Value] containing a [CoreValue::Native] and the appropriate type definition.
    fn boxed_to_datex_native_value(
        self: Box<Self>,
        cache: &mut SharedReferencesCache,
    ) -> Value;
}

/// A trait for dynamic equality comparison of types that implement `Any`.
pub trait DynEq: Any {
    fn dyn_eq(&self, other: &dyn Any) -> bool;
}

/// Default implementation of `DynEq` for all types that implement `Any`.
/// This implementation always returns `false`, indicating that the types are not equal.
impl<T> DynEq for T
where
    T: Any,
{
    default fn dyn_eq(&self, _other: &dyn Any) -> bool {
        false
    }
}

/// Specialized implementation of `DynEq` for types that implement both `Any` and `PartialEq`.
/// This implementation checks if the other type can be downcast to the same type and compares them for equality.
impl<T> DynEq for T
where
    T: Any + PartialEq,
{
    fn dyn_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<T>() == Some(self)
    }
}