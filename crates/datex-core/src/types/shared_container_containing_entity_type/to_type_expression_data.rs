use crate::ast::type_expressions::{IdentifierWithPointerAddress, TypeExpressionData};
use crate::traits::to_type_expression_data::ToTypeExpressionData;
use crate::types::shared_container_containing_entity_type::SharedContainerContainingEntityType;

impl ToTypeExpressionData for SharedContainerContainingEntityType {
    fn to_type_expression_data(&self) -> TypeExpressionData {
        let pointer_address = self.pointer_address();
        TypeExpressionData::IdentifierWithPointerAddress(
            IdentifierWithPointerAddress {
                name: self.entity_definition().name.clone(),
                pointer_address,
            },
        )
    }
}