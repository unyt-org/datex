use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::{dyn_eq::DynEq, try_clone::TryClone, value_access::ValueAccess},
};
use core::any::Any;
use crate::preludes::derive::DatexNativeOnlyStructural;
use crate::traits::convert_parts::{FromParts, IntoParts};
use crate::traits::convert_value_container::ConvertValueContainer;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
#[cfg(feature = "decompiler")]
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::types::entity_type::EntityType;
use crate::values::value::value_classification::{ValueClassification, ValueTag};

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
    /// The default implementation returns None, indicating that the value does not have an entity type.
    fn entity_type(&self, cache: &mut SharedReferencesCache) -> Option<EntityType> {
        None
    }
    
    /// Returns a [ValueTag] if the value has a tag.
    /// The default implementation returns None, indicating that the value does not have a tag.
    fn tag(&self) -> Option<ValueTag> {
        None
    }

    /// Returns the DATEX [ValueClassification] of the native value.
    /// This tries to resolve the entity type and tag, assuming at most one of them is present.
    fn classification(&self, cache: &mut SharedReferencesCache) -> ValueClassification {
        if let Some(entity_type) = self.entity_type(cache) {
            ValueClassification::Entity(entity_type)
        } else if let Some(tag) = self.tag() {
            ValueClassification::Tag(tag)
        } else {
            ValueClassification::None
        }
        // TODO: impl types?
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

    /// Returns the DATEX [EntityType] of the native value if it has an entity type.
    /// For structural types, this will return None.
    fn classification(&self, cache: &mut SharedReferencesCache) -> ValueClassification;
}