use crate::{
    prelude::*,
    shared_values::errors::AccessError,
    traits::value_access::ValueAccess,
    types::r#type::Type,
    values::value_container::{
        ValueContainer, borrowed_value_container::BorrowedValueContainer,
    },
};

impl ValueAccess for Type {
    fn try_get_property(
        &self,
        key: &ValueContainer,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        match self {
            Type::Entity(entity) => entity.try_get_property(key),
            _ => Err(AccessError::InvalidOperation(
                "Cannot get property".to_string(),
            )),
        }
    }
}
