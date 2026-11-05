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
                type_definition::receiver::ReceiverTypeDefinition,
            },
        },
    },
    values::core_values::receiver::Receiver,
};

impl<T> GetDatexType for Receiver<T>
where
    T: GetDatexType,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::Collection(CollectionTypeDefinition::Receiver(
                ReceiverTypeDefinition(Box::new(T::datex_type(memory))),
            ))
            .into(),
        )
    }
}
