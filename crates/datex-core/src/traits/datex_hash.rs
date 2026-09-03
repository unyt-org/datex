use core::hash::Hasher;
use crate::collections::default_hasher;

/// A trait for types that can be hashed using the default hasher
/// This trait is automatically implemented for all types that implement [core::hash::Hash].
pub trait DatexHash {
    fn datex_hash(&self) -> u64;
}

impl<T> DatexHash for T
where
    T: core::hash::Hash,
{
    fn datex_hash(&self) -> u64 {
        let mut hasher = default_hasher();
        self.hash(&mut hasher);
        hasher.finish()
    }
}