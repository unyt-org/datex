use crate::{
    preludes::derive::{
        CoreLibBaseTypeId, SharedReferencesCache, TypeDefinition,
    },
    traits::get_datex_type::GetDatexType,
    types::r#type::Type,
    values::value::Value,
};

impl GetDatexType for Value {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Any.into()).into(),
        )
    }
}
