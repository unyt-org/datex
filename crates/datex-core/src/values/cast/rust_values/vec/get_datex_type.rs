use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::get_datex_type::GetDatexType,
    types::type_definition::collection::{
        CollectionTypeDefinition,
        type_definition::list::ListCollectionTypeDefinition,
    },
};

impl<T> GetDatexType for Vec<T>
where
    T: GetDatexType,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::Collection(CollectionTypeDefinition::List(
                ListCollectionTypeDefinition(Box::new(T::datex_type(memory))),
            ))
            .into(),
        )
    }
}
