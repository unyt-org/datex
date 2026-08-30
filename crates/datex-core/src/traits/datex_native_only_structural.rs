use crate::preludes::derive::DatexNative;

/// Marker trait indicating that this value can be converted to a DATEX [Value]
/// without a cache since no entity type references must be resolved.
pub trait DatexNativeOnlyStructural: DatexNative {

}