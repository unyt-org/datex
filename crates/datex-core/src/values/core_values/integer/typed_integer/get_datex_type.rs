use crate::preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache};
use crate::traits::get_datex_type::GetDatexType;
use crate::types::r#type::Type;
use crate::values::core_values::integer::typed_integer::TypedInteger;

impl GetDatexType for TypedInteger {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Integer)
    }
}