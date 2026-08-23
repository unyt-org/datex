use crate::datex_proxy::DatexValueProxy;
use crate::values::borrowed_value_container::{AsBorrowed, BorrowedValueContainer};
use crate::values::core_values::native::DatexNative;

impl<'a, T> AsBorrowed<'a> for Vec<T>
where
    T: DatexNative + DatexValueProxy,
{
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::native_borrowed(self)
    }
}
