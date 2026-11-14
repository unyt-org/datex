use crate::{prelude::*, traits::datex_hash::DatexHash};
use core::hash::Hasher;

impl<T> DatexHash for Vec<T>
where
    T: DatexHash,
{
    fn datex_hash(&self, mut state: &mut dyn Hasher) {
        for value in self {
            value.datex_hash(&mut state);
        }
    }
}
