//! This module defines the representation for interface method call expressions in the AST.
use crate::{ast::expressions::DatexExpression, prelude::*};
/// Represents an interface method call expression.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceMethodCall {
    /// The target expression on which the method is called.
    pub target: DatexExpression,
    /// The name of the method being called.
    pub method_name: String,
    /// The arguments passed to the method.
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
