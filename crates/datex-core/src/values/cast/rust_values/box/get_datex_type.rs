use crate::{
    prelude::*, runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::get_datex_type::GetDatexType,
};

impl<T> GetDatexType for Box<T>
where
    T: GetDatexType,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        T::datex_type(memory)
    }
}
