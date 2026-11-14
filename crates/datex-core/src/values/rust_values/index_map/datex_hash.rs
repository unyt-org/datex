use crate::{random::RandomState, traits::datex_hash::DatexHash};
use core::hash::Hasher;
use indexmap::IndexMap;

impl<K, V> DatexHash for IndexMap<K, V, RandomState>
where
    K: DatexHash,
    V: DatexHash,
{
    fn datex_hash(&self, mut state: &mut dyn Hasher) {
        for (key, value) in self {
            key.datex_hash(&mut state);
            value.datex_hash(&mut state);
        }
    }
}
