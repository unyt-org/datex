use core::hash::Hasher;
use crate::traits::datex_hash::DatexHash;
use crate::prelude::*;

impl<T> DatexHash for Box<T>
where
    T: DatexHash + ?Sized,
{
    fn datex_hash(&self, mut state: &mut dyn Hasher) {
        (**self).datex_hash(&mut state);
    }
}