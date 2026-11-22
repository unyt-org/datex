use crate::{
    ast::expressions::DatexExpression, global::operators::UnaryOperator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub expression: DatexExpression,
}
