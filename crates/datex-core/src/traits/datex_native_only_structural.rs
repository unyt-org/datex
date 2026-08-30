use crate::traits::datex_native_structural::DatexNativeStructural;

/// Marker trait indicating that this value and any of its sub-values do not have an entity type.
/// This guarantees that [DatexNative::entity_type] will always return None for this value.
pub trait DatexNativeOnlyStructural: DatexNativeStructural {

}