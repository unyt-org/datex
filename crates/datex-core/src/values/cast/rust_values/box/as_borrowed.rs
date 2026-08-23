use crate::values::borrowed_value_container::{AsBorrowed, BorrowedValueContainer};
use crate::prelude::*;

impl<'a, T> AsBorrowed<'a> for Box<T>
where
    T: AsBorrowed<'a>,
{
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        (*self).as_borrowed()
    }
}
