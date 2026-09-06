use crate::{
    preludes::derive::DatexNative,
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
        get_datex_type::GetDatexType,
    },
};

/// `Option<T>` always implements [DatexNativeStructural].
impl<T: DatexNative + GetDatexType> DatexNativeStructural for Option<T> {}

/// If `T` implements [DatexNativeOnlyStructural], then `Option<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + GetDatexType> DatexNativeOnlyStructural
    for Option<T>
{
}
