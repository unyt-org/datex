use crate::preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache};
use crate::traits::get_datex_type::GetDatexType;
use crate::types::r#type::Type;
use crate::values::core_values::range::Range;

impl GetDatexType for Range {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Range)
    }
}