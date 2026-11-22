use crate::{ast::expressions::DatexExpression, prelude::*};

#[derive(Clone, Debug, PartialEq)]
pub struct Apply {
    pub base: DatexExpression,
    pub arguments: Vec<DatexExpression>,
}
