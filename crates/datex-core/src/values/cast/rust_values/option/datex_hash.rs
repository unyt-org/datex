use crate::preludes::derive::CoreValue;
use crate::traits::datex_hash::DatexHash;

impl<T> DatexHash for Option<T>
where
    T: DatexHash,
{
    fn datex_hash(&self) -> u64 {
        match self {
            Some(value) => value.datex_hash(),
            None => CoreValue::Null.datex_hash(),
        }
    }
}