use crate::{
    ast::expressions::DatexExpression, global::operators::BinaryOperator,
    types::r#type::Type,
};

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryOperation {
    pub operator: BinaryOperator,
    pub left: DatexExpression,
    pub right: DatexExpression,
    pub ty: Option<Type>,
}
