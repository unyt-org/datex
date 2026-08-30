use crate::preludes::derive::{SharedReferencesCache, Type};

// Returns the DATEX [Type] for the target
pub trait GetDatexType {
    fn datex_type(context: &mut SharedReferencesCache) -> Type;
    fn datex_type_without_cache() -> Type
    where
        Self: Sized,
    {
        Self::datex_type(&mut SharedReferencesCache::default())
    }
}