use crate::{
    traits::datex_hash::DatexHash, values::core_values::sender::Sender,
};
use core::hash::{Hash, Hasher};

impl<T> Hash for Sender<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // FIXME: senders / receivers should never be hashed to the same value
        state.write_u8(0xFF);
    }
}

impl<T> DatexHash for Sender<T>
where
    T: DatexHash,
{
    fn datex_hash(&self, state: &mut dyn Hasher) {
        // FIXME: senders / receivers should never be hashed to the same value
        state.write_u8(0xFF);
    }
}
