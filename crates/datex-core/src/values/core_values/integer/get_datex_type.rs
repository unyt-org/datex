use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::get_datex_type::GetDatexType, types::r#type::Type,
    values::core_values::integer::Integer,
};

impl GetDatexType for Integer {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Integer)
    }
}
