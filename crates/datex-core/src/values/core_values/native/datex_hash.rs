use crate::traits::datex_hash::{DatexHash};
use crate::values::core_values::native::NativeCoreValue;

impl DatexHash for NativeCoreValue {
    fn datex_hash(&self) -> u64 {
        self.value.datex_hash()
    }
}