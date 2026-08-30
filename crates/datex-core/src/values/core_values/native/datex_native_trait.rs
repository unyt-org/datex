use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::{dyn_eq::DynEq, try_clone::TryClone, value_access::ValueAccess},
};
use core::any::Any;
use crate::traits::convert_parts::{FromParts, IntoParts};
use crate::traits::convert_value_container::ConvertValueContainer;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
#[cfg(feature = "decompiler")]
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::types::entity_type::EntityType;

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

    /// Returns the DATEX [EntityType] of the native value if it has an entity type.
    /// For structural types, this will return None.
    fn entity_type(&self, cache: &mut SharedReferencesCache) -> Option<EntityType>;
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

    /// Returns the DATEX [EntityType] of the native value if it has an entity type.
    /// For structural types, this will return None.
    fn entity_type(&self, cache: &mut SharedReferencesCache) -> Option<EntityType>;
}