use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;

/// If `T` implements [DatexNativeOnlyStructural], then `Option<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural> DatexNativeOnlyStructural for Option<T> {}