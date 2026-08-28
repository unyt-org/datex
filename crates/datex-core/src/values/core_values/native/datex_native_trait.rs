use crate::{
    datex_proxy::DatexValueProxySerialize,
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::{dyn_eq::DynEq, try_clone::TryClone, value_access::ValueAccess},
    values::value::Value,
};
use core::any::Any;

// TODO: better solution than duplicate definition of trait for different feature flags?
#[cfg(feature = "decompiler")]
pub trait DatexNative:
    Any
    + DynEq
    + DatexValueProxySerialize
    + ValueAccess
    + TryClone
    + crate::traits::to_datex_expression_data::ToDatexExpressionData
{
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convert the boxed native value into a [Value] containing a [CoreValue::Native] and the appropriate type definition.
    fn boxed_to_datex_native_value(
        self: Box<Self>,
        cache: &mut SharedReferencesCache,
    ) -> Value;

    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Any.into()
    }
}

#[cfg(not(feature = "decompiler"))]
pub trait DatexNative:
    Any + DynEq + DatexValueProxySerialize + ValueAccess + TryClone
{
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convert the boxed native value into a [Value] containing a [CoreValue::Native] and the appropriate type definition.
    fn boxed_to_datex_native_value(
        self: Box<Self>,
        cache: &mut SharedReferencesCache,
    ) -> Value;

    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Any.into()
    }
}
