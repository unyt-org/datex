use crate::{
    prelude::*,
    preludes::derive::{SharedReferencesCache, Type, TypeDefinition},
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
