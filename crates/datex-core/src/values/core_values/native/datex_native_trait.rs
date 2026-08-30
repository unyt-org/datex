use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::{dyn_eq::DynEq, try_clone::TryClone, value_access::ValueAccess},
};
use core::any::Any;
use crate::traits::convert_parts::{FromParts, IntoParts};
use crate::traits::convert_value_container::ConvertValueContainer;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::traits::get_datex_type::GetDatexType;
#[cfg(feature = "decompiler")]
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::types::r#type::Type;

// TODO: better solution than duplicate definition of trait for different feature flags?
#[cfg(feature = "decompiler")]
pub trait DatexNative:
    Any +
    DynEq +
    FromParts +
    IntoParts +
    GetCoreLibTypeId +
    ValueAccess +
    TryClone +
    ConvertValueContainer +
    ToDatexExpressionData
{
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Returns the DATEX [Type] of the native value.
    fn value_datex_type(&self, cache: &mut SharedReferencesCache) -> Type;
}

default impl<T> DatexNative for T
where
    T:
        Any +
        DynEq +
        FromParts +
        IntoParts +
        GetCoreLibTypeId +
        ValueAccess +
        TryClone +
        ConvertValueContainer +
        ToDatexExpressionData +
        GetDatexType,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn value_datex_type(&self, cache: &mut SharedReferencesCache) -> Type {
        T::datex_type(cache)
    }
}

#[cfg(not(feature = "decompiler"))]
pub trait DatexNative:
    Any +
    DynEq +
    FromParts +
    IntoParts +
    GetCoreLibTypeId +
    ValueAccess +
    TryClone +
    ConvertValueContainer
{
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /// Returns the DATEX [Type] of the native value.
    fn datex_type(&self, cache: &mut SharedReferencesCache) -> Type;
}

#[cfg(not(feature = "decompiler"))]
default impl<T> DatexNative for T
where
    T:
    Any +
    DynEq +
    FromParts +
    IntoParts +
    GetCoreLibTypeId +
    ValueAccess +
    TryClone +
    ConvertValueContainer
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn value_datex_type(&self, cache: &mut SharedReferencesCache) -> Type {
        T::datex_type(cache)
    }
}