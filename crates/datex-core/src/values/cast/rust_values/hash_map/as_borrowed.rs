use crate::{
    collections::HashMap,
    datex_proxy::DatexValueProxy,
    values::{
        borrowed_value_container::{AsBorrowed, BorrowedValueContainer},
        core_values::native::DatexNative,
    },
};
use core::hash::Hash;

impl<'a, K, V> AsBorrowed<'a> for HashMap<K, V>
where
    K: DatexNative + DatexValueProxy + Eq + Hash,
    V: DatexNative + DatexValueProxy,
{
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::native_borrowed(self)
    }
}
