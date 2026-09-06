use crate::{
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
    },
    values::core_values::endpoint::Endpoint,
};

impl DatexNativeStructural for Endpoint {}
impl DatexNativeOnlyStructural for Endpoint {}
