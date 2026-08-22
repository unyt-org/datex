use crate::ast::expressions::{DatexExpressionData};
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::types::entities::entity_type_definition::EntityTypeDefinition;

impl ToDatexExpressionData for EntityTypeDefinition {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        todo!()
    }
}