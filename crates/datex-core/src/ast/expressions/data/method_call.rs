//! This module defines the representation for interface method call expressions in the AST.
use crate::{ast::expressions::DataExpression, prelude::*};
/// Represents an interface method call expression.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceMethodCall {
    /// The target expression on which the method is called.
    pub target: DataExpression,
    /// The name of the method being called.
    pub method_name: String,
    /// The arguments passed to the method.
    pub arguments: Vec<DataExpression>,
}

impl InterfaceMethodCall {
    pub fn new(
        target: DataExpression,
        method_name: String,
        arguments: Vec<DataExpression>,
    ) -> Self {
        InterfaceMethodCall {
            target,
            method_name,
            arguments,
        }
    }
}
