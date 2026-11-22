//! This module defines the representation for list expressions in the AST.
use crate::{ast::expressions::DataExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents a list expression.
#[derive(Clone, Debug, PartialEq)]
pub struct List {
    /// The items in the list expression.
    pub items: Vec<DataExpression>,
}

impl List {
    pub fn new(items: Vec<DataExpression>) -> Self {
        List { items }
    }
}

impl FromIterator<DataExpression> for List {
    fn from_iter<T: IntoIterator<Item = DataExpression>>(iter: T) -> Self {
        let items = iter.into_iter().collect();
        List { items }
    }
}
