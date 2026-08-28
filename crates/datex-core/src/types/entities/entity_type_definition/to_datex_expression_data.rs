use crate::{
    ast::expressions::DatexExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    types::entities::entity_type_definition::EntityTypeDefinition,
};

impl ToDatexExpressionData for EntityTypeDefinition {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        todo!()
    }
}
