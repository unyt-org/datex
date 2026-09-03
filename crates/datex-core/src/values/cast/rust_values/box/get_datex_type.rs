use crate::preludes::derive::{SharedReferencesCache, Type, TypeDefinition, UnionTypeDefinition};
use crate::traits::get_datex_type::GetDatexType;
use crate::prelude::*;

impl<T> GetDatexType for Box<T>
where
    T: GetDatexType,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        T::datex_type(memory)
    }
}