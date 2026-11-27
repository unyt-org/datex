//! This module defines the representation for interface method call expressions in the AST.
use crate::{ast::expressions::NormalExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents an interface method call expression.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceMethodCall {
    /// The target expression on which the method is called.
    pub target: NormalExpression,
    /// The name of the method being called.
    pub method_name: String,
    /// The arguments passed to the method.
    pub arguments: Vec<NormalExpression>,
}

impl InterfaceMethodCall {
    pub fn new(
        target: NormalExpression,
        method_name: String,
        arguments: Vec<NormalExpression>,
    ) -> Self {
        InterfaceMethodCall {
            target,
            method_name,
            arguments,
        }
    }
}
