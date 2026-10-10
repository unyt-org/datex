use crate::{
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    values::{
        borrowed_value_container::BorrowedValueContainer,
        value_container::ValueContainer,
    },
};

impl<'a> GetCoreLibTypeId for BorrowedValueContainer<'a> {}
