use crate::{
    traits::datex_hash::impl_datex_hash,
    values::{
        borrowed_value_container::BorrowedValueContainer,
        value_container::ValueContainer,
    },
};

impl_datex_hash!(BorrowedValueContainer<'_>);
