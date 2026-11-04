use crate::{
    prelude::*,
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
        get_datex_type::GetDatexType,
    },
    values::core_values::native::DatexNativeBase,
};
use crate::values::core_values::sender::Sender;

/// `Sender<T>` always implements [DatexNativeStructural].
impl<T: DatexNativeBase + GetDatexType + 'static> DatexNativeStructural
    for Sender<T>
{
}

/// If `T` implements [DatexNativeOnlyStructural], then `Sender<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + GetDatexType> DatexNativeOnlyStructural
    for Sender<T>
{
}
