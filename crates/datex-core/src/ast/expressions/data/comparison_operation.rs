use crate::{
    ast::expressions::DatexExpression, global::operators::ComparisonOperator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonOperation {
    pub operator: ComparisonOperator,
    pub left: DatexExpression,
    pub right: DatexExpression,
}
