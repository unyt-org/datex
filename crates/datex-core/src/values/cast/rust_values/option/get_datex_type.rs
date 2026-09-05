use crate::preludes::derive::{SharedReferencesCache, Type, TypeDefinition, UnionTypeDefinition};
use crate::traits::get_datex_type::GetDatexType;
use crate::prelude::*;

/// TODO: only wrap nested Option<Option<T>> into container. Single option can be mapped directly to X|null
impl<T> GetDatexType for Option<T>
where
    T: GetDatexType,
{
    /// Returns the container type definition for `Option<T>`, which is a union of `null` and the type definition of `T`,
    /// wrapped in a container
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        let inner_type = T::datex_type(memory);
        Type::Definition(
            TypeDefinition::Box(Box::new(
                TypeDefinition::Union(UnionTypeDefinition(vec![
                    Type::NULL,
                    inner_type,
                ]))
                    .into(),
            ))
                .into(),
        )
    }
}