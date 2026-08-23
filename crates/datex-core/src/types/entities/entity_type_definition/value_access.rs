use crate::shared_values::errors::{AccessError};
use crate::traits::value_access::ValueAccess;
use crate::types::entities::entity_type_definition::EntityTypeDefinition;
use crate::values::value::ValueContainerOrBorrowedValue;
use crate::values::value_container::value_key::BorrowedValueKey;

impl ValueAccess for EntityTypeDefinition {
    fn try_get_property(&self, key: BorrowedValueKey) -> Result<ValueContainerOrBorrowedValue<'_>, AccessError> {
        todo!()
    }
}