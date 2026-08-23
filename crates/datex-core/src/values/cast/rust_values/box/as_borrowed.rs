use crate::values::borrowed_value_container::{AsBorrowed, BorrowedValueContainer};

impl<'a, T> AsBorrowed<'a> for Box<T>
where
    T: AsBorrowed<'a>,
{
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        (*self).as_borrowed()
    }
}
