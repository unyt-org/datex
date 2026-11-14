use crate::{
    prelude::*,
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
        get_datex_type::GetDatexType,
    },
    values::core_values::native::DatexNativeBase,
};

/// `Vec<T>` always implements [DatexNativeStructural].
impl<T: DatexNativeBase + GetDatexType + 'static> DatexNativeStructural
    for Vec<T>
{
}

/// If `T` implements [DatexNativeOnlyStructural], then `Vec<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + GetDatexType> DatexNativeOnlyStructural
    for Vec<T>
{
}
