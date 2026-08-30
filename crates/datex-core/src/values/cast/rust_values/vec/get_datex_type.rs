use crate::preludes::derive::{SharedReferencesCache, Type, TypeDefinition};
use crate::traits::get_datex_type::GetDatexType;
use crate::types::type_definition::collection::CollectionTypeDefinition;
use crate::types::type_definition::collection::type_definition::list::ListCollectionTypeDefinition;

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