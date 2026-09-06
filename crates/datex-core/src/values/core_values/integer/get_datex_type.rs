use crate::{
    preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache},
    traits::get_datex_type::GetDatexType,
    types::r#type::Type,
    values::core_values::integer::Integer,
};

impl GetDatexType for Integer {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Integer)
    }
}
