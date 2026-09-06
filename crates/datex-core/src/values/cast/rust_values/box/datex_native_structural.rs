use crate::{
    prelude::*,
    preludes::derive::StaticClassification,
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
        get_datex_type::GetDatexType,
    },
};

/// If `T` implements [DatexNativeStructural], then `Box<T>` also implements [DatexNativeStructural].
impl<T: DatexNativeStructural + GetDatexType + StaticClassification>
    DatexNativeStructural for Box<T>
{
}

/// If `T` implements [DatexNativeOnlyStructural], then `Box<T>` also implements [DatexNativeOnlyStructural].
impl<T: DatexNativeOnlyStructural + GetDatexType + StaticClassification>
    DatexNativeOnlyStructural for Box<T>
{
}
