use crate::traits::datex_hash::{DatexHash};
use crate::values::core_values::native::NativeCoreValue;

impl DatexHash for NativeCoreValue {
    fn datex_hash(&self, hasher: &mut dyn core::hash::Hasher) {
        self.value.datex_hash(hasher)
    }
}