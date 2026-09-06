use crate::{prelude::*, traits::datex_hash::DatexHash};
use core::hash::Hasher;

impl<T> DatexHash for Box<T>
where
    T: DatexHash + ?Sized,
{
    fn datex_hash(&self, mut state: &mut dyn Hasher) {
        (**self).datex_hash(&mut state);
    }
}
