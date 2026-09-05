pub mod classification;
pub mod datex_hash;
pub mod datex_native;
mod try_from_core_value;

use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    preludes::derive::CoreLibTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::{
        convert_parts::{FromParts, IntoParts},
        datex_hash::impl_datex_hash,
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
        get_core_lib_type_id::GetCoreLibTypeId,
        get_datex_type::GetDatexType,
        value_access::ValueAccess,
    },
    types::r#type::Type,
    values::{
        borrowed_value_container::{
            AsBorrowed, AsBorrowedMut, BorrowedValueContainer,
        },
        core_values::native::DatexNative,
    },
};
use core::time::Duration;
mod to_instructions;
#[cfg(feature = "ast")]
mod to_datex_expression_data {
    use crate::{
        ast::expressions::DatexExpressionData,
        traits::to_datex_expression_data::ToDatexExpressionData,
        values::core_values::integer::Integer,
    };
    use core::time::Duration;

    impl ToDatexExpressionData for Duration {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            // TODO: use amount once implemented
            DatexExpressionData::Integer(Integer::from(self.as_millis()))
        }
    }
}

impl ValueAccess for Duration {}
impl FromParts for Duration {}
impl IntoParts for Duration {}
impl GetCoreLibTypeId for Duration {}
impl GetDatexType for Duration {}

impl DatexNativeStructural for Duration {}
impl DatexNativeOnlyStructural for Duration {}
