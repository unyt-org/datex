use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::get_datex_type::GetDatexType,
    types::{r#type::Type, type_definition::TypeDefinition},
};

impl GetDatexType for Type {
    fn datex_type(_cache: &mut SharedReferencesCache) -> Type {
        TypeDefinition::CoreType(CoreLibBaseTypeId::Type.into()).into()
    }
}
