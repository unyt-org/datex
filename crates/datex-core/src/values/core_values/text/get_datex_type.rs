use crate::preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache, Text};
use crate::traits::get_datex_type::GetDatexType;
use crate::types::r#type::Type;

impl GetDatexType for Text {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Text)
    }
}