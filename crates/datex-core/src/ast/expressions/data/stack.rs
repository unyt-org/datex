use crate::{
    ast::expressions::DatexExpression, global::stack_index::StackIndex,
};

#[derive(Clone, Debug, PartialEq)]
pub struct StackAssignment {
    pub index: StackIndex,
    pub expression: DatexExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StackListAssignment {
    pub expression: DatexExpression,
}
