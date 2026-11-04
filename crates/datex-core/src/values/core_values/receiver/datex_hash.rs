use crate::{traits::datex_hash::DatexHash};
use core::hash::Hasher;
use core::hash::Hash;
use crate::values::core_values::receiver::Receiver;
impl<T> Hash for Receiver<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // FIXME: senders / receivers should never be hashed to the same value
        state.write_u8(0xFE);
    }
}

impl<T> DatexHash for Receiver<T>
where
    T: DatexHash,
{
    fn datex_hash(&self, mut state: &mut dyn Hasher) {
        // FIXME: senders / receivers should never be hashed to the same value
        state.write_u8(0xFE);
    }
}
