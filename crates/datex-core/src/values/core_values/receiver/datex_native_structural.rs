use crate::{
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
        get_datex_type::GetDatexType,
    },
    values::core_values::{native::DatexNativeBase, receiver::Receiver},
};

/// `Receiver<T>` always implements [DatexNativeStructural].
impl<T: DatexNativeBase + GetDatexType + 'static> DatexNativeStructural
    for Receiver<T>
{
}

/// If `T` implements [DatexNativeOnlyStructural], then `Receiver<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + GetDatexType> DatexNativeOnlyStructural
    for Receiver<T>
{
}
