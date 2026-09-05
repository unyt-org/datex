use crate::preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache, TypeDefinition};
use crate::traits::get_datex_type::GetDatexType;
use crate::types::r#type::Type;
use crate::values::value::Value;

impl GetDatexType for Value {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Any.into()).into(),
        )
    }
}