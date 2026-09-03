use crate::{
    traits::{dyn_eq::DynEq, try_clone::TryClone, value_access::ValueAccess},
};
use core::any::Any;
use crate::traits::classification::Classification;
use crate::traits::convert_parts::{FromParts, IntoParts};
use crate::traits::convert_value_container::ConvertValueContainer;
use crate::traits::datex_hash::DatexHash;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::traits::get_datex_type::GetDatexType;
#[cfg(feature = "ast")]
use crate::traits::to_datex_expression_data::ToDatexExpressionData;


#[cfg(feature = "ast")]
pub trait DatexNativeBase: 
    ConvertValueContainer + 
    GetDatexType + 
    IntoParts + 
    FromParts + 
    Classification +
    GetCoreLibTypeId +
    DatexHash +
    ToDatexExpressionData {}
#[cfg(feature = "ast")]
impl <T> DatexNativeBase for T where T: 
    ConvertValueContainer + 
    GetDatexType + 
    IntoParts + 
    FromParts + 
    Classification +
    GetCoreLibTypeId +
    DatexHash +
    ToDatexExpressionData {}

#[cfg(not(feature = "ast"))]
pub trait DatexNativeBase:
    ConvertValueContainer +
    GetDatexType +
    IntoParts +
    FromParts +
    Classification +
    GetCoreLibTypeId +
    DatexHash
{}
#[cfg(not(feature = "ast"))]
impl <T> DatexNativeBase for T where T: 
    ConvertValueContainer +
    GetDatexType +
    IntoParts +
    FromParts +
    Classification +
    GetCoreLibTypeId +
    DatexHash
{}

// TODO: better solution than duplicate definition of trait for different feature flags?
#[cfg(feature = "ast")]
pub trait DatexNative:
    Any +
    DynEq +
    DatexHash +
    FromParts +
    IntoParts +
    GetCoreLibTypeId +
    ValueAccess +
    TryClone +
    ConvertValueContainer +
    Classification +
    ToDatexExpressionData +
{
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[cfg(not(feature = "ast"))]
pub trait DatexNative:
    Any +
    DynEq +
    DatexHash +
    FromParts +
    IntoParts +
    GetCoreLibTypeId +
    ValueAccess +
    TryClone +
    ConvertValueContainer +
    Classification
{
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}