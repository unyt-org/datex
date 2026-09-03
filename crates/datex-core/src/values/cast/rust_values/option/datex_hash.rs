use core::hash::Hasher;
use crate::preludes::derive::CoreValue;
use crate::traits::datex_hash::DatexHash;

impl<T> DatexHash for Option<T>
where
    T: DatexHash,
{
    fn datex_hash(&self, mut state: &mut dyn Hasher) {
        match self {
            Some(value) => {
                value.datex_hash(&mut state);
            }
            None => CoreValue::Null.datex_hash(&mut state),
        }
    }
}