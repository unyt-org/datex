use crate::{
    preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache},
    traits::get_datex_type::GetDatexType,
    types::r#type::Type,
    values::core_values::decimal::Decimal,
};

impl GetDatexType for Decimal {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Decimal)
    }
}
