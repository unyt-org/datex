use crate::{
    ast::type_expressions::{IdentifierWithPointerAddress, TypeExpressionData},
    traits::to_type_expression_data::ToTypeExpressionData,
    types::entity_type::EntityType,
};

impl ToTypeExpressionData for EntityType {
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
