use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::get_datex_type::GetDatexType,
    types::{r#type::Type, type_definition::TypeDefinition},
    values::value_container::ValueContainer,
};

impl GetDatexType for ValueContainer {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Any.into()).into(),
        )
    }
}
