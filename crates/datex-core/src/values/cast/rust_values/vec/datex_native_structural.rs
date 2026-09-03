use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::traits::get_datex_type::GetDatexType;
use crate::values::core_values::native::DatexNativeBase;
use crate::prelude::*;

/// `Vec<T>` always implements [DatexNativeStructural].
impl<T: DatexNativeBase + GetDatexType + 'static> DatexNativeStructural for Vec<T> {}

/// If `T` implements [DatexNativeOnlyStructural], then `Vec<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + GetDatexType> DatexNativeOnlyStructural for Vec<T> {}