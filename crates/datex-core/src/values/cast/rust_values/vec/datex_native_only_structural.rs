use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;

/// If `T` implements [DatexNativeOnlyStructural], then `Vec<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural> DatexNativeOnlyStructural for Vec<T> {}