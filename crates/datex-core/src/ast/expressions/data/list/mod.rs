//! This module defines the representation for list expressions in the AST.
use crate::{ast::expressions::NormalExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents a list expression.
#[derive(Clone, Debug, PartialEq)]
pub struct List {
    /// The items in the list expression.
    pub items: Vec<NormalExpression>,
}

impl List {
    pub fn new(items: Vec<NormalExpression>) -> Self {
        List { items }
    }
}

impl FromIterator<NormalExpression> for List {
    fn from_iter<T: IntoIterator<Item =NormalExpression>>(iter: T) -> Self {
        let items = iter.into_iter().collect();
        List { items }
    }
}
