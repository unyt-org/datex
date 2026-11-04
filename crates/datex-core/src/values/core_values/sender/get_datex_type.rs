use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::get_datex_type::GetDatexType,
    types::{
        r#type::Type,
        type_definition::{
            TypeDefinition,
            collection::{
                CollectionTypeDefinition,
            },
        },
    },
};
use crate::values::core_values::sender::Sender;
use crate::types::type_definition::collection::type_definition::sender::SenderTypeDefinition;

impl<T> GetDatexType for Sender<T>
where
    T: GetDatexType,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::Collection(CollectionTypeDefinition::Sender(
                SenderTypeDefinition(Box::new(T::datex_type(memory))),
            ))
            .into(),
        )
    }
}
