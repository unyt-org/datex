use crate::{
    preludes::derive::{ConvertCoreValue, DatexNative},
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
    },
};

/// `Option<T>` always implements [DatexNativeStructural].
impl<T: DatexNative + ConvertCoreValue> DatexNativeStructural for Option<T> {}

/// If `T` implements [DatexNativeOnlyStructural], then `Option<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + ConvertCoreValue> DatexNativeOnlyStructural
    for Option<T>
{
}
