use crate::{ast::expressions::DatexExpression, prelude::*};

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceMethodCall {
    pub target: DatexExpression,
    pub method_name: String,
    pub arguments: Vec<DatexExpression>,
}

impl InterfaceMethodCall {
    pub fn new(
        target: DatexExpression,
        method_name: String,
        arguments: Vec<DatexExpression>,
    ) -> Self {
        InterfaceMethodCall {
            target,
            method_name,
            arguments,
        }
    }
}
