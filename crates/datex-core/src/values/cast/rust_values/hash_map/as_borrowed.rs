use crate::collections::HashMap;
use core::hash::Hash;
use crate::datex_proxy::DatexValueProxy;
use crate::values::borrowed_value_container::{AsBorrowed, BorrowedValueContainer};
use crate::values::core_values::native::DatexNative;

impl<'a, K, V> AsBorrowed<'a> for HashMap<K, V>
where
    K: DatexNative + DatexValueProxy + Eq + Hash,
    V: DatexNative + DatexValueProxy,
{
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::native_borrowed(self)
    }
}
