use core::hash::Hasher;
use crate::traits::datex_hash::DatexHash;
use crate::prelude::*;

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