use crate::{
    preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache, Text},
    traits::get_datex_type::GetDatexType,
    types::r#type::Type,
};

impl GetDatexType for Text {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Text)
    }
}
