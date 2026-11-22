use core::ops::{self};

use crate::types::r#type::Type;

pub trait Expression<T> {
    fn new(data: T, span: ops::Range<usize>) -> Self;

    fn data(&self) -> &T;

    fn data_mut(&mut self) -> &mut T;

    fn span(&self) -> &ops::Range<usize>;

    fn ty(&self) -> Option<&Type>;
}
