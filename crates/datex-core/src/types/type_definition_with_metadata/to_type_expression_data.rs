use crate::ast::type_expressions::TypeExpressionData;
use crate::traits::to_type_expression_data::ToTypeExpressionData;
use crate::types::type_definition_with_metadata::TypeDefinitionWithMetadata;

impl ToTypeExpressionData for TypeDefinitionWithMetadata {
    fn to_type_expression_data(&self) -> TypeExpressionData {
        // TODO: handle type metadata
        self.definition.to_type_expression_data()
    }
}