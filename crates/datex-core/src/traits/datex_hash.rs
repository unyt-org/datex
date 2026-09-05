use core::hash::Hasher;
use core::hash::Hash;

/// A trait for types that can be hashed using the default hasher
/// This trait is automatically implemented for all types that implement [core::hash::Hash].
pub trait DatexHash {
    fn datex_hash(&self, state: &mut dyn Hasher);
}

pub macro impl_datex_hash($t:ty) {
    impl DatexHash for $t {
        fn datex_hash(&self, mut state: &mut dyn Hasher) {
            self.hash(&mut state);
        }
    }
}