use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::get_datex_type::GetDatexType;
use crate::prelude::*;
use crate::traits::datex_native_structural::DatexNativeStructural;

/// If `T` implements [DatexNativeStructural], then `Box<T>` also implements [DatexNativeStructural].
impl<T: DatexNativeStructural + GetDatexType> DatexNativeStructural for Box<T> {}

/// If `T` implements [DatexNativeOnlyStructural], then `Box<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + GetDatexType> DatexNativeOnlyStructural for Box<T> {}