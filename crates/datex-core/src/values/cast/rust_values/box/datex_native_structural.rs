use crate::{
    prelude::*,
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
        get_datex_type::GetDatexType,
    },
};
use crate::traits::classification::Classification;

/// If `T` implements [DatexNativeStructural], then `Box<T>` also implements [DatexNativeStructural].
impl<T: DatexNativeStructural + GetDatexType + Classification>
    DatexNativeStructural for Box<T>
{
}

/// If `T` implements [DatexNativeOnlyStructural], then `Box<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + GetDatexType + Classification>
    DatexNativeOnlyStructural for Box<T>
{
}
