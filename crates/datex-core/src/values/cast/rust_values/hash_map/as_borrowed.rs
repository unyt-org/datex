use crate::{
    collections::HashMap,
    values::{
        borrowed_value_container::{AsBorrowed, BorrowedValueContainer},
        core_values::native::DatexNative,
    },
};
use core::hash::Hash;
use crate::traits::get_datex_type::GetDatexType;

impl<'a, K, V> AsBorrowed<'a> for HashMap<K, V>
where
    K: DatexNative + GetDatexType + Eq + Hash,
    V: DatexNative + GetDatexType,
{
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::native_borrowed_only_structural(self)
    }
}
