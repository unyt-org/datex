use crate::{
    ast::type_expressions::TypeExpressionData,
    traits::to_type_expression_data::ToTypeExpressionData, types::r#type::Type,
};

impl ToTypeExpressionData for Type {
    fn to_type_expression_data(&self) -> TypeExpressionData {
        match self {
            Type::Entity(container) => container.to_type_expression_data(),
            Type::Definition(definition) => {
                definition.to_type_expression_data()
            }
        }
    }
}
