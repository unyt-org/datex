use crate::{
    preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache},
    traits::get_datex_type::GetDatexType,
    types::r#type::Type,
    values::core_values::native::NativeCoreValue,
};

impl GetDatexType for NativeCoreValue {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Any)
    }
}
