use crate::traits::datex_hash::DatexHash;

impl<T> DatexHash for Box<T>
where
    T: DatexHash,
{
    fn datex_hash(&self) -> u64 {
        (**self).datex_hash()
    }
}