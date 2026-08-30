use crate::values::core_values::native::DatexNative;

/// Marker trait indicating that this value can be converted to a DATEX [Value]
/// without a cache since no entity type references must be resolved.
/// This guarantees that [DatexNative::entity_type] will always return None for this value.
pub trait DatexNativeOnlyStructural: DatexNative {

}