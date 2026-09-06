use crate::{
    preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache},
    traits::get_datex_type::GetDatexType,
    types::r#type::Type,
    values::core_values::list::List,
};

impl GetDatexType for List {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::List)
    }
}
