use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    traits::get_datex_type::GetDatexType, types::r#type::Type,
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

impl GetDatexType for TypedDecimal {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Decimal)
    }
}
