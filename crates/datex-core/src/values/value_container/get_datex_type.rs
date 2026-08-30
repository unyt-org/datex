use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::{r#type::Type, type_definition::TypeDefinition},
    values::value_container::ValueContainer,
};
use crate::traits::get_datex_type::GetDatexType;

impl GetDatexType for ValueContainer {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Any.into()).into(),
        )
    }
}
