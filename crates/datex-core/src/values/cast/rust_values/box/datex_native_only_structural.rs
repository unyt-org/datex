use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::get_datex_type::GetDatexType;
use crate::prelude::*;

/// If `T` implements [DatexNativeOnlyStructural], then `Box<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + GetDatexType> DatexNativeOnlyStructural for Box<T> {}