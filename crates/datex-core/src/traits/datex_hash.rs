use crate::collections::DefaultHasher;

/// A trait for types that can be hashed using the default hasher
/// This trait is automatically implemented for all types that implement [core::hash::Hash].
pub trait DatexHash {
    fn datex_hash(&self, hasher: &mut DefaultHasher); // TODO
}

pub macro impl_datex_hash($t:ty) {
    impl DatexHash for $t {
        fn datex_hash(&self) -> u64 {
            use crate::collections::default_hasher;
            use core::hash::Hasher;
            use core::hash::Hash;

            let mut hasher = default_hasher();
            self.hash(&mut hasher);
            hasher.finish()
        }
    }
}