use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::traits::get_datex_type::GetDatexType;

/// If `T` implements [DatexNativeStructural], then `Vec<T>` also implements [DatexNativeStructural].
impl<T: DatexNativeStructural + GetDatexType> DatexNativeStructural for Vec<T> {}

/// If `T` implements [DatexNativeOnlyStructural], then `Vec<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + GetDatexType> DatexNativeOnlyStructural for Vec<T> {}