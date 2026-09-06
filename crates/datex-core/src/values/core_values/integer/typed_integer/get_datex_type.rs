use crate::{
    preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache},
    traits::get_datex_type::GetDatexType,
    types::r#type::Type,
    values::core_values::integer::typed_integer::TypedInteger,
};

impl GetDatexType for TypedInteger {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Integer)
    }
}
