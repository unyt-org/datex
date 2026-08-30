use crate::{
    prelude::*,
    values::{
        borrowed_value_container::{AsBorrowed, BorrowedValueContainer},
        core_values::native::DatexNative,
    },
};

impl<'a, T> AsBorrowed<'a> for Vec<T>
where
    T: DatexNative,
{
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::native_borrowed_only_structural(self)
    }
}
