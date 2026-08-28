use crate::values::borrowed_value_container::{
    AsBorrowed, BorrowedValueContainer,
};

impl<'a, T> AsBorrowed<'a> for Option<T>
where
    T: AsBorrowed<'a>,
{
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        match self {
            Some(value) => value.as_borrowed(),
            None => todo!("borrow none option"),
        }
    }
}
